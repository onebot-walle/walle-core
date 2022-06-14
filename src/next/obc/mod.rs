// use core::future::Future;

// use tokio::task::JoinHandle;

// use crate::WalleResult;

use super::{ActionContext, EHACtrait, OneBot, Static};
use crate::utils::{Echo, EchoS};
use crate::{error::WalleResult, utils::ProtocolItem};
use async_trait::async_trait;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock, Mutex};
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
impl<E, A, R, ECAH, const V: u8>
    EHACtrait<E, A, R, OneBot<E, A, R, ECAH, Self, V>, crate::config::ImplConfig> for ImplOBC<E>
where
    E: ProtocolItem + Static + Clone,
    A: ProtocolItem + Static,
    R: ProtocolItem + Static + Debug,
    ECAH: Static,
{
    async fn start(
        &self,
        ob: &Arc<OneBot<E, A, R, ECAH, Self, V>>,
        config: crate::config::ImplConfig,
    ) -> WalleResult<(
        Vec<JoinHandle<()>>,
        mpsc::UnboundedReceiver<ActionContext<A, R>>,
    )> {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<ActionContext<A, R>>();
        let mut tasks = vec![];
        #[cfg(feature = "websocket")]
        {
            self.ws(ob, config.websocket, action_tx.clone(), &mut tasks)
                .await?;
            self.wsr(ob, config.websocket_rev, action_tx, &mut tasks)
                .await?;
        }
        Ok((tasks, action_rx))
    }
    async fn handle(&self, event: E, _ob: &OneBot<E, A, R, ECAH, Self, V>) {
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

type EchoMap<R> = Arc<Mutex<HashMap<EchoS, mpsc::UnboundedSender<Echo<R>>>>>;
type BotMap<A> = Arc<RwLock<HashMap<String, mpsc::UnboundedSender<Echo<A>>>>>;

#[derive(Clone)]
pub struct AppOBC<A, R> {
    pub echos: EchoMap<R>,
    pub bots: BotMap<A>,
}

impl<A, R> AppOBC<A, R> {
    pub fn new() -> Self {
        Self {
            echos: Arc::new(Mutex::new(HashMap::new())),
            bots: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}
