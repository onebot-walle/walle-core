use std::{sync::Arc, time::Duration};

use tokio::task::JoinHandle;

use crate::{app::CustomOneBot, config::WebSocketClient};

pub async fn run<E, A, R>(
    config: &WebSocketClient,
    ob: Arc<CustomOneBot<E, A, R>>,
) -> JoinHandle<()>
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    let config = config.clone();
    tokio::spawn(async move {
        loop {
            if let Ok(ws_stream) = crate::comms::ws_utils::try_connect(&config).await {
                super::websocket_loop(ws_stream, ob.clone()).await;
            }
            tokio::time::sleep(Duration::from_secs(config.reconnect_interval as u64)).await;
        }
    })
}
