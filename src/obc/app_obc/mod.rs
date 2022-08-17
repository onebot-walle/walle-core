use std::collections::HashSet;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Arc;

use super::OBC;
use crate::ah::GetSelfs;
use crate::structs::Selft;
use crate::util::{Echo, EchoInner, EchoS, GetSelf, ProtocolItem};
use crate::{ActionHandler, EventHandler, GetStatus, OneBot};
use crate::{WalleError, WalleResult};

use async_trait::async_trait;
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
    pub(crate) echos: EchoMap<R>,    // echo channel sender 暂存 Map
    pub(crate) seq: AtomicU64,       // 用于生成 echo
    pub(crate) bots: Arc<BotMap<A>>, // Bot action channel map
}

impl<A, R> AppOBC<A, R> {
    pub fn new() -> Self {
        Default::default()
    }
}

impl<A, R> Default for AppOBC<A, R> {
    fn default() -> Self {
        Self {
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

#[async_trait]
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
    async fn call(&self, action: A) -> WalleResult<R> {
        match self.bots.get_bot(&action.get_self()) {
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
}

#[derive(Debug)]
pub(crate) struct BotMap<A> {
    pub(crate) tx_seq: AtomicUsize,
    pub(crate) bots: DashMap<Selft, (String, Vec<mpsc::UnboundedSender<Echo<A>>>)>,
    pub(crate) txs: DashMap<usize, (mpsc::UnboundedSender<Echo<A>>, HashSet<Selft>)>,
}

impl<A> Default for BotMap<A> {
    fn default() -> Self {
        Self {
            tx_seq: AtomicUsize::default(),
            bots: DashMap::default(),
            txs: DashMap::default(),
        }
    }
}

impl<A> BotMap<A> {
    fn new_connect(&self) -> (usize, mpsc::UnboundedReceiver<Echo<A>>) {
        let seq = self.tx_seq.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::unbounded_channel();
        self.txs.insert(seq, (tx, HashSet::default()));
        (seq, rx)
    }
    fn connect_closs(&self, tx_seq: &usize) {
        if let Some(selfts) = self.txs.remove(tx_seq) {
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
    fn connect_update(&self, tx_seq: &usize, mut selfts: HashSet<Selft>, implt: &str) {
        let mut get_selfts = self.txs.get_mut(tx_seq).unwrap();
        let mut need_remove = vec![];
        let tx = get_selfts.value().0.clone();
        for selft in &get_selfts.value().1 {
            if selfts.contains(selft) {
                selfts.remove(selft);
            } else {
                need_remove.push(selft.clone())
            }
        }
        for selft in need_remove {
            get_selfts.1.remove(&selft);
            if let Some(mut bots) = self.bots.get_mut(&selft) {
                bots.value_mut().1.retain(|htx| !htx.same_channel(&tx));
                if bots.1.is_empty() {
                    drop(bots);
                    self.bots.remove(&selft);
                }
            }
            info!(
                target: OBC,
                "Bot disconnected: {}-{}", selft.platform, selft.user_id
            );
        }
        for selft in selfts {
            self.bots
                .entry(selft.clone())
                .or_insert((implt.to_owned(), vec![]))
                .1
                .push(tx.clone());
            get_selfts.1.insert(selft.clone());
            info!(
                target: OBC,
                "New Bot connected: {}-{}", selft.platform, selft.user_id
            );
        }
    }
    fn get_bot(&self, bot: &Selft) -> Option<Vec<mpsc::UnboundedSender<Echo<A>>>> {
        self.bots.get(bot).as_deref().cloned().map(|v| v.1)
    }
    fn selfts(&self) -> Vec<Selft> {
        self.bots.iter().map(|i| i.key().clone()).collect()
    }
}

#[async_trait]
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

#[async_trait]
impl<A, R> GetStatus for AppOBC<A, R>
where
    A: Send + Sync,
    R: Send + Sync,
{
    async fn is_good(&self) -> bool {
        true
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
        map.txs
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
    map.connect_update(&0, HashSet::from([self0.clone()]), "");
    assert_eq!(map.bots.get(&self0).unwrap().1.len(), 1);
    assert!(map.txs.get(&0).unwrap().1.len() == 1);
    map.connect_update(&0, HashSet::from([self1.clone()]), "");
    assert!(map.bots.get(&self0).is_none());
    assert!(map.txs.get(&0).unwrap().1.len() == 1);
    assert_eq!(map.bots.get(&self1).unwrap().1.len(), 1);
    assert!(map.get_bot(&self1).is_some());
}
