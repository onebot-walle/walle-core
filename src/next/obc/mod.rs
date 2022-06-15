use super::{ECAHtrait, EHACtrait, OneBot, Static};
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
#[cfg(feature = "websocket")]
mod impl_ws;

#[derive(Clone)]
pub struct ImplOBC<E> {
    pub platform: String,
    pub r#impl: String,
    pub(crate) event_tx: tokio::sync::broadcast::Sender<E>,
    pub(crate) hb_tx: tokio::sync::broadcast::Sender<crate::StandardEvent>,
}

#[async_trait]
impl<E, A, R, ECAH, const V: u8> EHACtrait<E, A, R, ECAH, V> for ImplOBC<E>
where
    E: ProtocolItem + Clone,
    A: ProtocolItem,
    R: ProtocolItem + Debug,
    ECAH: ECAHtrait<E, A, R, Self, V> + Static,
{
    type Config = crate::config::ImplConfig;
    async fn ehac_start(
        &self,
        ob: &Arc<OneBot<ECAH, Self, V>>,
        config: crate::config::ImplConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>> {
        let mut tasks = vec![];
        #[cfg(feature = "websocket")]
        {
            tasks.extend(self.ws(ob, config.websocket).await?.into_iter());
            tasks.extend(self.wsr(ob, config.websocket_rev).await?.into_iter());
        }
        Ok(tasks)
    }
    async fn handle_event(&self, event: E, _ob: &OneBot<ECAH, Self, V>) {
        self.event_tx.send(event).ok();
    }
}

impl<E> ImplOBC<E> {
    pub fn new(r#impl: String, platform: String) -> Self
    where
        E: Clone,
    {
        let (event_tx, _) = tokio::sync::broadcast::channel(1024); //todo
        let (hb_tx, _) = tokio::sync::broadcast::channel(1024);
        Self {
            platform,
            r#impl,
            event_tx,
            hb_tx,
        }
    }
}

impl<C> ImplOBC<crate::BaseEvent<C>> {
    pub async fn new_event_with_time(
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

    pub async fn new_event(&self, content: C, self_id: String) -> crate::BaseEvent<C> {
        self.new_event_with_time(crate::utils::timestamp_nano_f64(), content, self_id)
            .await
    }
}

use dashmap::DashMap;

type EchoMap<R> = Arc<DashMap<EchoS, oneshot::Sender<R>>>;
type BotMap<A> = Arc<DashMap<String, mpsc::UnboundedSender<Echo<A>>>>;

pub struct AppOBC<A, R> {
    pub echos: EchoMap<R>,
    pub bots: BotMap<A>,
    seq: AtomicU64,
}

impl<A, R> AppOBC<A, R> {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn next_seg(&self) -> EchoS {
        EchoS(Some(EchoInner::S(
            self.seq
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                .to_string(),
        )))
    }
}

impl<A, R> AppOBC<A, R>
where
    A: SelfId,
{
    pub async fn _handle_action(&self, action: A) -> WalleResult<R> {
        match self.bots.get(&action.self_id()) {
            Some(action_tx) => {
                let (tx, rx) = oneshot::channel();
                let seq = self.next_seg();
                self.echos.insert(seq.clone(), tx);
                action_tx.send(seq.pack(action)).map_err(|e| {
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
}

impl<A, R> Default for AppOBC<A, R> {
    fn default() -> Self {
        Self {
            echos: Arc::new(DashMap::new()),
            bots: Arc::new(DashMap::new()),
            seq: AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl<E, A, R, EHAC, const V: u8> ECAHtrait<E, A, R, EHAC, V> for AppOBC<A, R>
where
    E: ProtocolItem + Clone + SelfId,
    A: ProtocolItem + SelfId + ActionType,
    R: ProtocolItem + Debug,
    EHAC: EHACtrait<E, A, R, Self, V> + Static,
{
    type Config = crate::config::AppConfig;
    async fn ecah_start(
        &self,
        ob: &Arc<OneBot<Self, EHAC, V>>,
        config: crate::config::AppConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>> {
        let mut tasks = vec![];
        #[cfg(feature = "websocket")]
        {
            tasks.extend(self.wsr(ob, config.websocket_rev).await?.into_iter());
            tasks.extend(self.ws(ob, config.websocket).await?.into_iter());
        }
        Ok(tasks)
    }
    async fn handle_action(&self, action: A, _ob: &OneBot<Self, EHAC, V>) -> WalleResult<R> {
        self._handle_action(action).await
    }
}
