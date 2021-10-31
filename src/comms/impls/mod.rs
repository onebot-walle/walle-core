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
    sender: crate::impls::CustomActionSender<A, R>,
) where
    E: Clone + serde::Serialize + Send + 'static,
    A: serde::de::DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: serde::Serialize + std::fmt::Debug + Send + 'static,
{
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;
    use tracing::error;

    // let (mut sink, mut stream) = ws_stream.split();
    let (resp_sender, mut resp_receiver) = tokio::sync::mpsc::channel(1024);
    loop {
        tokio::select! {
            event_result = listener.recv() => {
                match event_result {
                    Ok(event) => {
                        let event = serde_json::to_string(&event).unwrap();
                        ws_stream.send(Message::Text(event)).await.unwrap();
                    }
                    Err(_) => panic!(),
                }
            }
            resp = resp_receiver.recv() => {
                let resp = serde_json::to_string(&resp).unwrap();
                ws_stream.send(Message::Text(resp)).await.unwrap();
            }
            data_option = ws_stream.next() => {
                if let Some(data) = data_option {
                    match data {
                        Ok(message) => {
                            match serde_json::from_str(&message.to_string()) {
                                Ok(action) => {
                                    sender.send((action, crate::impls::CustomARSS::Mpsc(resp_sender.clone())))
                                    .await.unwrap();
                                }
                                Err(_) => error!("Receive illegal action {}", message.to_string()),
                            }
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
    // let sink_join = tokio::spawn(async move {
    //     loop {
    //         let s = tokio::select! {
    //             event = listener.recv() => {
    //                 if let Ok(event) = event {
    //                     serde_json::to_string(&event).unwrap()
    //                 }
    //                 else { panic!() }
    //             }
    //             resp = resp_receiver.recv() => { serde_json::to_string(&resp).unwrap() }
    //         };
    //         sink.send(Message::Text(s)).await.unwrap();
    //     }
    // });
    // let stream_join = tokio::spawn(async move {
    //     loop {
    //         if let Some(data) = stream.next().await {
    //             if let Ok(message) = data {
    //                 match serde_json::from_str(&message.to_string()) {
    //                     Ok(action) => {
    //                         sender
    //                             .send((action, crate::impls::CustomARSS::Mpsc(resp_sender.clone())))
    //                             .await
    //                             .unwrap();
    //                     }
    //                     Err(_) => {}
    //                 }
    //             }
    //         }
    //     }
    // });
    // (sink_join, stream_join)
}

use super::util;
