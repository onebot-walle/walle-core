use std::time::Duration;

use tokio::task::JoinHandle;

use crate::config::WebSocketRev;

pub async fn run<E, A, R>(config: &WebSocketRev) -> JoinHandle<()> {
    let config = config.clone();
    tokio::spawn(async move {
        while let Some(ws_stream) = super::util::try_connect(&config).await {
            super::websocker_loop::<crate::Event>(ws_stream).await;
            tokio::time::sleep(Duration::from_secs(config.reconnect_interval as u64)).await;
        }
    })
}
