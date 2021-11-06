#[cfg(feature = "http")]
mod http;
#[cfg(feature = "http")]
mod http_webhook;
#[cfg(feature = "websocket")]
mod websocket;
#[cfg(feature = "websocket")]
mod websocket_rev;

#[cfg(feature = "http")]
pub use http::run as http_run;
#[cfg(feature = "http")]
pub use http_webhook::Client as WebhookClient;
#[cfg(feature = "websocket")]
pub use websocket::run as websocket_run;
#[cfg(feature = "websocket")]
pub use websocket_rev::run as websocket_rev_run;

#[cfg(feature = "websocket")]
async fn websocket_loop<E, A, R>(
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    mut listener: crate::impls::CustomEventListner<E>,
    handler: crate::impls::ArcActionHandler<A, R>,
) where
    E: Clone + serde::Serialize + Send + 'static,
    A: serde::de::DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: serde::Serialize + std::fmt::Debug + Send + 'static,
{
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;
    use tracing::error;

    use crate::utils::Echo;

    // let (mut sink, mut stream) = ws_stream.split();
    let (resp_sender, mut resp_receiver) = tokio::sync::mpsc::unbounded_channel();
    loop {
        tokio::select! {
            event_result = listener.recv() => {
                match event_result {
                    Ok(event) => {
                        let event = serde_json::to_string(&event).unwrap();
                        if let Err(e) = ws_stream.send(Message::Text(event)).await {
                            error!(target: "Walle-core", "ws disconnect with error {}", e);
                            return;
                        };
                    }
                    Err(_) => panic!(),
                }
            }
            resp = resp_receiver.recv() => {
                if let Some(resp) = resp {
                    let resp = serde_json::to_string(&resp).unwrap();
                    ws_stream.send(Message::Text(resp)).await.unwrap();
                }
            }
            data_option = ws_stream.next() => {
                if let Some(data) = data_option {
                    match data {
                        Ok(message) => {
                            match serde_json::from_str::<Echo<A>>(&message.to_string()) {
                                Ok(action) => {
                                    let action_handler = handler.clone();
                                    let sender = resp_sender.clone();
                                    tokio::spawn(async move {
                                        let (action, echo) = action.unpack();
                                        let resp = action_handler.handle(action).await;
                                        let resp = echo.pack(resp);
                                        sender.send(resp).unwrap();
                                    });
                                }
                                Err(_) => error!(target: "Walle-core", "Receive illegal action {}", message.to_string()),
                            }
                        },
                        Err(e) => {
                            error!(target: "Walle-core", "ws disconnect with error {}", e);
                            return;
                        }
                    }
                }
            }
        }
    }
}
