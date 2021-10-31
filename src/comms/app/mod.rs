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
async fn websocket_loop<E, A, R>(
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    event_handler: crate::app::ArcEventHandler<E>,
    mut action_listener: tokio::sync::broadcast::Receiver<A>,
    action_resp_handler: crate::app::ArcARHandler<R>,
) where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    use futures_util::{SinkExt, StreamExt};
    use serde::Deserialize;
    use tokio_tungstenite::tungstenite::Message;
    use tracing::error;

    use crate::event::CustomEvent;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum ReceiveItem<E, R> {
        Event(CustomEvent<E>),
        Resp(R),
    }

    loop {
        tokio::select! {
            action_result = action_listener.recv() => {
                if let Ok(action) = action_result {
                    let action = serde_json::to_string(&action).unwrap();
                    if let Err(e) = ws_stream.send(Message::text(action)).await {
                        error!("ws disconnect with error {}", e);
                        return;
                    }
                }
            }
            option_data = ws_stream.next() => {
                if let Some(data) = option_data {
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
        }
    }
}
