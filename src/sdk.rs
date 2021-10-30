use std::sync::Arc;

use crate::{Action, ActionResps, Event};

type ArcEventHandler<E> = Arc<dyn crate::handle::EventHandler<E> + Send + Sync>;
type ArcARHandler<R> = Arc<dyn crate::handle::ActionRespHandler<R> + Send + Sync>;
type CustomActionBroadcaster<A> = tokio::sync::broadcast::Sender<A>;

#[allow(unused)]
pub struct CustomOneBot<E, A, R> {
    pub config: crate::config::SdkConfig,
    event_handler: ArcEventHandler<E>,
    action_broadcaster: CustomActionBroadcaster<A>,
    action_resp_handler: ArcARHandler<R>,
}

impl<E, A, R> CustomOneBot<E, A, R> {
    #[allow(unused)]
    pub async fn run(&self) {
        use tracing::info;

        #[cfg(feature = "websocket")]
        if !self.config.websocket.is_empty() {
            info!("Running WebSocket");
            for websocket in &self.config.websocket {
                crate::comms::sdk::websocket_run::<Event, Action, ActionResps>(websocket).await;
            }
        }
    }
}
