use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

use crate::{config::SdkConfig, Action, ActionResps, Event};

pub(crate) type ArcEventHandler<E> = Arc<dyn crate::handle::EventHandler<E> + Send + Sync>;
pub(crate) type ArcARHandler<R> = Arc<dyn crate::handle::ActionRespHandler<R> + Send + Sync>;
type CustomActionBroadcaster<A> = tokio::sync::broadcast::Sender<A>;

#[allow(unused)]
pub type OneBot = CustomOneBot<Event, Action, ActionResps>;

#[allow(unused)]
pub struct CustomOneBot<E, A, R> {
    pub config: SdkConfig,
    event_handler: ArcEventHandler<E>,
    action_broadcaster: CustomActionBroadcaster<A>,
    action_resp_handler: ArcARHandler<R>,
    running: AtomicBool,
}

impl<E, A, R> CustomOneBot<E, A, R>
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + Send,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    pub fn new(
        config: SdkConfig,
        event_handler: ArcEventHandler<E>,
        action_resp_handler: ArcARHandler<R>,
    ) -> Self {
        let (action_broadcaster, _) = tokio::sync::broadcast::channel(1024);
        Self {
            config,
            event_handler,
            action_broadcaster,
            action_resp_handler,
            running: AtomicBool::default(),
        }
    }

    #[allow(unused)]
    pub async fn run(&self) {
        use tracing::info;

        #[cfg(feature = "websocket")]
        if let Some(websocket) = &self.config.websocket {
            info!("Running WebSocket");
            crate::comms::sdk::websocket_run(
                websocket,
                self.event_handler.clone(),
                self.action_resp_handler.clone(),
            )
            .await;
            self.running.swap(true, Ordering::SeqCst);
        }

        #[cfg(feature = "websocket")]
        if let (Some(websocket_rev), false) = (
            &self.config.websocket_rev,
            self.running.load(Ordering::SeqCst),
        ) {
            info!("Running WebSocket");
            crate::comms::sdk::websocket_rev_run(
                websocket_rev,
                self.event_handler.clone(),
                self.action_resp_handler.clone(),
            )
            .await;
            self.running.swap(true, Ordering::SeqCst);
        }
    }
}
