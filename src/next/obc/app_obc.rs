use std::sync::{atomic::AtomicU64, Arc};

use crate::action::ActionType;
use crate::next::{ActionHandler, EventHandler, OneBotExt, Static};
use crate::utils::{Echo, EchoInner, EchoS, ProtocolItem, SelfId};
use crate::{WalleError, WalleResult};

use async_trait::async_trait;
use dashmap::DashMap;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tracing::warn;

pub(crate) type EchoMap<R> = Arc<DashMap<EchoS, oneshot::Sender<R>>>;
pub(crate) type BotMap<A> = Arc<BotMapInner<A>>;

/// OneBotConnect 应用端实现
///
/// AppOBC impl ActionHandler 接收 Action 并外发处理
///
/// Event 泛型要求实现 Clone + SelfId trait
/// Action 泛型要求实现 SelfId + ActionType trait
pub struct AppOBC<A, R> {
    pub echos: EchoMap<R>,
    pub bots: BotMap<A>,
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
            bots: Arc::new(Default::default()),
        }
    }
}

#[async_trait]
impl<E, A, R, OB> ActionHandler<E, A, R, OB> for AppOBC<A, R>
where
    E: ProtocolItem + Clone + SelfId,
    A: ProtocolItem + SelfId + ActionType,
    R: ProtocolItem,
    OB: EventHandler<E, A, R, OB> + OneBotExt + Static,
{
    type Config = crate::config::AppConfig;
    async fn ecah_start(
        &self,
        ob: &Arc<OB>,
        config: crate::config::AppConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>> {
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
    async fn handle_action(&self, action: A, _ob: &OB) -> WalleResult<R> {
        self.bots.handle_action(action, &self.echos).await
    }
}

pub struct BotMapInner<A>(
    DashMap<String, Vec<mpsc::UnboundedSender<Echo<A>>>>,
    AtomicU64,
);

impl<A> Default for BotMapInner<A> {
    fn default() -> Self {
        Self(DashMap::new(), AtomicU64::new(0))
    }
}

impl<A> BotMapInner<A> {
    pub fn next_seg(&self) -> EchoS {
        EchoS(Some(EchoInner::S(
            self.1
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                .to_string(),
        )))
    }
    pub async fn handle_action<R>(&self, action: A, echo_map: &EchoMap<R>) -> WalleResult<R>
    where
        A: SelfId,
    {
        match self.0.get_mut(&action.self_id()) {
            Some(action_txs) => {
                let (tx, rx) = oneshot::channel();
                let seq = self.next_seg();
                echo_map.insert(seq.clone(), tx);
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
    pub fn ensure_tx(&self, bot_id: &str, tx: &mpsc::UnboundedSender<Echo<A>>) {
        self.0
            .entry(bot_id.to_string())
            .or_default()
            .push(tx.clone());
    }
    pub fn remove_bot(&self, bot_id: &str, tx: &mpsc::UnboundedSender<Echo<A>>) {
        if let Some(mut txs) = self.0.get_mut(bot_id) {
            for i in 0..txs.len() {
                if tx.same_channel(&txs[i]) {
                    txs.remove(i);
                }
            }
        };
    }
}
