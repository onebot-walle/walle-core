#[cfg(feature = "websocket")]
mod websocket;
#[cfg(feature = "websocket")]
mod websocket_rev;

#[cfg(feature = "websocket")]
pub(crate) use websocket::run as websocket_run;
#[cfg(feature = "websocket")]
pub(crate) use websocket_rev::run as websocket_rev_run;

use super::util;

#[cfg(feature = "websocket")]
async fn websocket_loop<E, R>(
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    event_handler: crate::sdk::ArcEventHandler<E>,
    action_resp_handler: crate::sdk::ArcARHandler<R>,
) where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    use futures_util::StreamExt;
    use serde::Deserialize;
    use tracing::error;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum ReceiveItem<E, R> {
        Event(E),
        Resp(R),
    }

    while let Some(data) = ws_stream.next().await {
        match data {
            Ok(message) => match serde_json::from_str::<ReceiveItem<E, R>>(&message.to_string()) {
                Ok(item) => match item {
                    ReceiveItem::Event(e) => {
                        let handler = event_handler.clone();
                        tokio::spawn(async move { handler.handle(e).await });
                    }
                    ReceiveItem::Resp(r) => {
                        let handler = action_resp_handler.clone();
                        tokio::spawn(async move { handler.handle(r).await });
                    }
                },
                Err(_) => error!("receive illegal event {:?}", message.to_string()),
            },
            Err(e) => {
                error!("ws disconnect with error {}", e);
                return;
            }
        }
    }
}
