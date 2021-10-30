use std::time::Duration;

use serde::{de::DeserializeOwned, Serialize};
use tokio::task::JoinHandle;

use crate::config::WebSocketRev;

pub async fn run<E, A, R>(
    config: &WebSocketRev,
    broadcaster: crate::impls::CustomEventBroadcaster<E>,
    sender: crate::impls::CustomActionSender<A, R>,
) -> JoinHandle<()>
where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    let config = config.clone();
    tokio::spawn(async move {
        while let Some(ws_stream) = super::util::try_connect(&config).await {
            super::websocket_loop(ws_stream, broadcaster.subscribe(), sender.clone()).await;
            tokio::time::sleep(Duration::from_secs(config.reconnect_interval as u64)).await;
        }
    })
}
