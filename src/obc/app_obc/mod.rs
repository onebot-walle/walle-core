use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use super::OBC;
use crate::ah::GetSelfs;
use crate::prelude::{Bot, Version};
use crate::structs::Selft;
use crate::util::{Echo, EchoInner, EchoS, GetSelf, ProtocolItem};
use crate::{ActionHandler, EventHandler, GetStatus, GetVersion, OneBot};
use crate::{WalleError, WalleResult};

use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tracing::{info, warn};

#[cfg(feature = "http")]
mod app_http;
#[cfg(feature = "websocket")]
mod app_ws;

pub(crate) type EchoMap<R> = Arc<DashMap<EchoS, oneshot::Sender<R>>>;

/// OneBotConnect 应用端实现
///
/// AppOBC impl ActionHandler 接收 Action 并外发处理
///
/// Event 泛型要求实现 Clone + SelfId trait
/// Action 泛型要求实现 SelfId + ActionType trait
pub struct AppOBC<A, R> {
    pub(crate) _block_meta_event: AtomicBool, //todo
    pub(crate) echos: EchoMap<R>,             // echo channel sender 暂存 Map
    pub(crate) seq: AtomicU64,                // 用于生成 echo
    pub(crate) bots: Arc<BotMap<A>>,          // Bot action channel map
}

impl<A, R> AppOBC<A, R> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn block_meta_event(&self, b: bool) {
        self._block_meta_event.swap(b, Ordering::Relaxed);
    }
}

impl<A, R> Default for AppOBC<A, R> {
    fn default() -> Self {
        Self {
            _block_meta_event: AtomicBool::new(true),
            echos: Arc::new(DashMap::new()),
            seq: AtomicU64::default(),
            bots: Arc::new(Default::default()),
        }
    }
}

impl<A, R> AppOBC<A, R> {
    pub(crate) fn next_seg(&self) -> EchoS {
        EchoS(Some(EchoInner::S(
            self.seq
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                .to_string(),
        )))
    }
}

impl<E, A, R> ActionHandler<E, A, R> for AppOBC<A, R>
where
    E: ProtocolItem + Clone + GetSelf,
    A: ProtocolItem + GetSelf,
    R: ProtocolItem,
{
    type Config = crate::config::AppConfig;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: crate::config::AppConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        let mut tasks = vec![];
        #[cfg(feature = "websocket")]
        {
            self.wsr(ob, config.websocket_rev, &mut tasks).await?;
            self.ws(ob, config.websocket, &mut tasks).await?;
        }
        #[cfg(feature = "http")]
        {
            self.webhook(ob, config.http_webhook, &mut tasks).await?;
            self.http(ob, config.http, &mut tasks).await?;
        }
        Ok(tasks)
    }
    async fn call<AH, EH>(&self, action: A, _ob: &Arc<OneBot<AH, EH>>) -> WalleResult<R>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        match self.bots.get_bot_tx(&action.get_self()) {
            Some(action_txs) => {
                let (tx, rx) = oneshot::channel();
                let seq = self.next_seg();
                self.echos.insert(seq.clone(), tx);
                action_txs
                    .first()
                    .unwrap() //todo
                    .send(seq.pack(action))
                    .map_err(|e| {
                        warn!(target: super::OBC, "send action error: {}", e);
                        WalleError::Other(e.to_string())
                    })?;
                match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
                    Ok(Ok(res)) => Ok(res),
                    Ok(Err(e)) => {
                        warn!(target: super::OBC, "resp recv error: {:?}", e);
                        Err(WalleError::Other(e.to_string()))
                    }
                    Err(_) => {
                        warn!(target: super::OBC, "resp timeout");
                        Err(WalleError::Other("resp timeout".to_string()))
                    }
                }
            }
            None => {
                warn!(target: super::OBC, "bot not found");
                Err(WalleError::BotNotExist)
            }
        }
    }
    async fn before_call_event<AH, EH>(
        &self,
        event: E,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> WalleResult<E> {
        if self._block_meta_event.load(Ordering::Relaxed) {
            use core::any::Any;
            let event: Box<dyn Any> = Box::new(event.clone());
            if let Ok(ty) = event.downcast::<crate::event::Event>().map(|e| e.ty) {
                if &ty == "meta" {
                    return Err(WalleError::Other("blocked".to_string()));
                }
            }
        }
        Ok(event)
    }
}

#[derive(Debug)]
pub(crate) struct BotMap<A> {
    /// 登记获取连接序列号
    pub(crate) conn_seq: AtomicUsize,
    /// 根据 bot 的 self 获取其 impl 字段和所有的 action_tx
    ///
    /// value: (implt, action_tx)
    pub(crate) bots: DashMap<Selft, (String, Vec<mpsc::UnboundedSender<Echo<A>>>)>,
    /// 根据连接序列号获取其 action_tx 和 所有 bot self
    ///
    /// value: (action_tx, selfts)
    pub(crate) conns: DashMap<usize, (mpsc::UnboundedSender<Echo<A>>, HashSet<Selft>)>,
}

impl<A> Default for BotMap<A> {
    fn default() -> Self {
        Self {
            conn_seq: AtomicUsize::default(),
            bots: DashMap::default(),
            conns: DashMap::default(),
        }
    }
}

impl<A> BotMap<A> {
    /// 登记一个新链接，返回新链接的 conn_seq，并返回一个接收 Echo<A> 的 Receiver
    fn new_connect(&self) -> (usize, mpsc::UnboundedReceiver<Echo<A>>) {
        let seq = self.conn_seq.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::unbounded_channel();
        self.conns.insert(seq, (tx, HashSet::default()));
        (seq, rx)
    }
    /// 根据 conn_seq 关闭一个链接，并移除所有相关的 bot 的 action_tx
    fn connect_closs(&self, tx_seq: &usize) {
        if let Some(selfts) = self.conns.remove(tx_seq) {
            for selft in selfts.1 .1 {
                let mut bot = self.bots.get_mut(&selft).unwrap();
                bot.value_mut()
                    .1
                    .retain(|htx| !htx.same_channel(&selfts.1 .0));
                if bot.value().1.is_empty() {
                    drop(bot);
                    self.bots.remove(&selft);
                }
            }
        }
    }
    /// 更新一个连接的 bot 列表
    fn connect_update(&self, tx_seq: &usize, bots: Vec<Bot>, implt: &str) {
        let mut get = self.conns.get_mut(tx_seq).unwrap();
        let tx = get.0.clone();
        let selfts = &mut get.1;
        for bot in bots {
            match (bot.online, selfts.contains(&bot.selft)) {
                (true, false) => {
                    selfts.insert(bot.selft.clone());
                    self.bots
                        .entry(bot.selft.clone())
                        .or_insert((implt.to_string(), vec![]))
                        .1
                        .push(tx.clone());
                    info!(
                        target: OBC,
                        "New Bot connected: {}-{}", bot.selft.platform, bot.selft.user_id
                    );
                }
                (false, true) => {
                    selfts.remove(&bot.selft);
                    if let Some(mut bots) = self.bots.get_mut(&bot.selft) {
                        bots.value_mut().1.retain(|htx| !htx.same_channel(&tx));
                        if bots.1.is_empty() {
                            drop(bots);
                            self.bots.remove(&bot.selft);
                        }
                    }
                    info!(
                        target: OBC,
                        "Bot disconnected: {}-{}", bot.selft.platform, bot.selft.user_id
                    );
                }
                _ => {}
            }
        }
    }
    /// 获取一个 bot 的 action_tx
    fn get_bot_tx(&self, bot: &Selft) -> Option<Vec<mpsc::UnboundedSender<Echo<A>>>> {
        self.bots.get(bot).as_deref().cloned().map(|v| v.1)
    }
    /// 获取所有 bot
    fn selfts(&self) -> Vec<Selft> {
        self.bots.iter().map(|i| i.key().clone()).collect()
    }
}

impl<A, R> GetSelfs for AppOBC<A, R>
where
    A: Send + Sync,
    R: Send + Sync,
{
    async fn get_selfs(&self) -> Vec<Selft> {
        self.bots.selfts()
    }
    async fn get_impl(&self, selft: &Selft) -> String {
        self.bots
            .bots
            .get(selft)
            .map(|v| v.value().0.clone())
            .unwrap_or_default()
    }
}

impl<A, R> GetStatus for AppOBC<A, R>
where
    A: Send + Sync,
    R: Send + Sync,
{
    async fn is_good(&self) -> bool {
        true
    }
}

impl<A, R> GetVersion for AppOBC<A, R> {
    fn get_version(&self) -> Version {
        Version {
            implt: super::OBC.to_owned(),
            version: crate::VERSION.to_owned(),
            onebot_version: "12".to_owned(),
        }
    }
}

#[test]
fn test_bot_map() {
    let map = BotMap::<crate::action::Action>::default();
    let (seq, _) = map.new_connect();
    assert_eq!(seq, 0);
    let (seq, _) = map.new_connect();
    assert_eq!(seq, 1);
    assert_eq!(
        map.conns
            .iter()
            .map(|i| i.key().clone())
            .collect::<HashSet<_>>(),
        HashSet::from([1, 0])
    );
    let self0 = Selft {
        platform: "".to_owned(),
        user_id: "0".to_owned(),
    };
    let self1 = Selft {
        platform: "".to_owned(),
        user_id: "1".to_owned(),
    };
    map.connect_update(
        &0,
        vec![Bot {
            selft: self0.clone(),
            online: true,
        }],
        "",
    );
    assert_eq!(map.bots.get(&self0).unwrap().1.len(), 1);
    assert!(map.conns.get(&0).unwrap().value().1.len() == 1);
    assert!(map.bots.get(&self1).is_none());
    assert!(map.get_bot_tx(&self0).is_some());
    assert!(map.get_bot_tx(&self1).is_none());
}
