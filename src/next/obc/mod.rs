use super::{ActionHandler, EventHandler, OneBotExt, Static};
use crate::action::ActionType;
use crate::error::{WalleError, WalleResult};
use crate::utils::ProtocolItem;
use crate::utils::{Echo, EchoInner, EchoS};
use crate::SelfId;
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;
use tracing::warn;

const OBC: &str = "Walle-OBC";

#[cfg(feature = "http")]
mod app_http;
#[cfg(feature = "websocket")]
mod app_ws;
#[cfg(feature = "http")]
mod impl_http;
#[cfg(feature = "websocket")]
mod impl_ws;

pub struct ImplOBC<E> {
    pub self_id: std::sync::RwLock<String>,
    pub platform: String,
    pub r#impl: String,
    pub(crate) event_tx: tokio::sync::broadcast::Sender<E>,
    pub(crate) hb_tx: tokio::sync::broadcast::Sender<crate::StandardEvent>,
}

#[async_trait]
impl<E, A, R, OB> EventHandler<E, A, R, OB> for ImplOBC<E>
where
    E: ProtocolItem + Clone,
    A: ProtocolItem,
    R: ProtocolItem + Debug,
    OB: ActionHandler<E, A, R, OB> + OneBotExt + Static,
{
    type Config = crate::config::ImplConfig;
    async fn ehac_start(
        &self,
        ob: &Arc<OB>,
        config: crate::config::ImplConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>> {
        let mut tasks = vec![];
        #[cfg(feature = "websocket")]
        {
            self.ws(ob, config.websocket, &mut tasks).await?;
            self.wsr(ob, config.websocket_rev, &mut tasks).await?;
        }
        #[cfg(feature = "http")]
        {
            self.http(ob, config.http, &mut tasks).await?;
            self.webhook(ob, config.http_webhook, &mut tasks).await?;
        }
        Ok(tasks)
    }
    async fn handle_event(&self, event: E, _ob: &OB) {
        self.event_tx.send(event).ok();
    }
}

impl<E> ImplOBC<E> {
    pub fn new(self_id: String, r#impl: String, platform: String) -> Self
    where
        E: Clone,
    {
        let (event_tx, _) = tokio::sync::broadcast::channel(1024); //todo
        let (hb_tx, _) = tokio::sync::broadcast::channel(1024);
        Self {
            self_id: std::sync::RwLock::new(self_id),
            platform,
            r#impl,
            event_tx,
            hb_tx,
        }
    }
    pub fn get_self_id(&self) -> String {
        self.self_id.read().unwrap().clone()
    }
    pub fn set_self_id(&self, self_id: &str) {
        *self.self_id.write().unwrap() = self_id.to_string();
    }
}

impl<C> ImplOBC<crate::BaseEvent<C>> {
    pub fn new_event_with_time(
        &self,
        time: f64,
        content: C,
        self_id: String,
    ) -> crate::BaseEvent<C> {
        crate::BaseEvent {
            id: crate::utils::new_uuid(),
            r#impl: self.r#impl.clone(),
            platform: self.platform.clone(),
            self_id,
            time,
            content,
        }
    }

    pub fn new_event(&self, content: C, self_id: String) -> crate::BaseEvent<C> {
        self.new_event_with_time(crate::utils::timestamp_nano_f64(), content, self_id)
    }
}

use dashmap::DashMap;

type EchoMap<R> = Arc<DashMap<EchoS, oneshot::Sender<R>>>;
type BotMap<A> = Arc<BotMapInner<A>>;

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
    R: ProtocolItem + Debug,
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
                        warn!(target: OBC, "send action error: {}", e);
                        WalleError::Other(e.to_string())
                    })?;
                match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
                    Ok(Ok(res)) => Ok(res),
                    Ok(Err(e)) => {
                        warn!(target: OBC, "resp recv error: {:?}", e);
                        Err(WalleError::Other(e.to_string()))
                    }
                    Err(_) => {
                        warn!(target: OBC, "resp timeout");
                        Err(WalleError::Other("resp timeout".to_string()))
                    }
                }
            }
            None => {
                warn!(target: OBC, "bot not found");
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
