#[cfg(feature = "websocket")]
mod websocket;

#[cfg(feature = "websocket")]
pub(crate) use websocket::run as websocket_run;

use super::util;

#[cfg(feature = "websocket")]
async fn websocker_loop<E>(mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>)
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    use futures_util::StreamExt;
    use tracing::{error, info};

    while let Some(data) = ws_stream.next().await {
        if let Ok(message) = data {
            match serde_json::from_str::<E>(&message.to_string()) {
                Ok(event) => info!("receive event {:?}", event),
                Err(_) => error!("receive illegal event {:?}", message.to_string()),
            }
        }
    }
}
