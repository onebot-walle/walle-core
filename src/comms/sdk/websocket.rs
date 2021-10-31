use std::time::Duration;

use tokio::task::JoinHandle;

use crate::config::WebSocketRev;

pub async fn run<E, R>(
    config: &WebSocketRev,
    event_handler: crate::sdk::ArcEventHandler<E>,
    action_resp_handler: crate::sdk::ArcARHandler<R>,
) -> JoinHandle<()>
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    let config = config.clone();
    tokio::spawn(async move {
        loop {
            if let Some(ws_stream) = super::util::try_connect(&config).await {
                super::websocket_loop(
                    ws_stream,
                    event_handler.clone(),
                    action_resp_handler.clone(),
                )
                .await;
            }
            tokio::time::sleep(Duration::from_secs(config.reconnect_interval as u64)).await;
        }
    })
}
