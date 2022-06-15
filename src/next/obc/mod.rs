use super::{ECAHtrait, EHACtrait, OneBotExt, Static};
use crate::utils::{Echo, EchoInner, EchoS};
use crate::{error::WalleResult, utils::ProtocolItem};
use async_trait::async_trait;
use std::fmt::Debug;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

const OBC: &str = "Walle-OBC";

#[cfg(feature = "websocket")]
mod app_ws;
#[cfg(feature = "websocket")]
mod impl_ws;

#[derive(Clone)]
pub struct ImplOBC<E> {
    pub self_id: Arc<tokio::sync::RwLock<String>>,
    pub platform: String,
    pub r#impl: String,
    pub(crate) event_tx: tokio::sync::broadcast::Sender<E>,
    pub(crate) hb_tx: tokio::sync::broadcast::Sender<crate::StandardEvent>,
}

#[async_trait]
impl<E, A, R, OB> EHACtrait<E, A, R, OB, crate::config::ImplConfig> for ImplOBC<E>
where
    E: ProtocolItem + Static + Clone,
    A: ProtocolItem + Static,
    R: ProtocolItem + Static + Debug,
    OB: Static,
{
    async fn ehac_start<C0>(
        &self,
        ob: &Arc<OB>,
        config: crate::config::ImplConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        OB: ECAHtrait<E, A, R, OB, C0> + OneBotExt,
    {
        let mut tasks = vec![];
        #[cfg(feature = "websocket")]
        {
            tasks.extend(self.ehac_start(ob, config.websocket).await?.into_iter());
            tasks.extend(self.ehac_start(ob, config.websocket_rev).await?.into_iter());
        }
        Ok(tasks)
    }
    async fn handle_event(&self, event: E, _ob: &OB) {
        self.event_tx.send(event).ok();
    }
}

impl<E> ImplOBC<E> {
    pub fn new(r#impl: String, platform: String, self_id: String) -> Self
    where
        E: Clone,
    {
        let (event_tx, _) = tokio::sync::broadcast::channel(1024); //todo
        let (hb_tx, _) = tokio::sync::broadcast::channel(1024);
        Self {
            self_id: Arc::new(tokio::sync::RwLock::new(self_id)),
            platform,
            r#impl,
            event_tx,
            hb_tx,
        }
    }

    pub async fn set_self_id(&self, self_id: String) {
        *self.self_id.write().await = self_id;
    }
}

impl<C> ImplOBC<crate::BaseEvent<C>> {
    pub async fn new_event_with_time(&self, time: f64, content: C) -> crate::BaseEvent<C> {
        crate::BaseEvent {
            id: crate::utils::new_uuid(),
            r#impl: self.r#impl.clone(),
            platform: self.platform.clone(),
            self_id: self.self_id.read().await.clone(),
            time,
            content,
        }
    }

    pub async fn new_event(&self, content: C) -> crate::BaseEvent<C> {
        self.new_event_with_time(crate::utils::timestamp_nano_f64(), content)
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

impl<A, R> Default for AppOBC<A, R> {
    fn default() -> Self {
        Self {
            echos: Arc::new(DashMap::new()),
            bots: Arc::new(DashMap::new()),
            seq: AtomicU64::new(0),
        }
    }
}
