use serde::{de::DeserializeOwned, Serialize};

#[cfg(any(feature = "http", feature = "websocket"))]
pub enum ContentTpye {
    Json,
    MsgPack,
}

#[cfg(any(feature = "http", feature = "websocket"))]
impl ContentTpye {
    pub fn new(s: &str) -> Option<Self> {
        match s {
            "application/json" => Some(Self::Json),
            "application/msgpack" => Some(Self::MsgPack),
            _ => None,
        }
    }
}

#[cfg(feature = "websocket")]
use tokio::{net::TcpStream, task::JoinHandle};

#[cfg(all(feature = "websocket", feature = "impl"))]
pub(crate) async fn websocket_loop<E, A, R>(
    ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>,
    mut listener: crate::impls::CustomEventListner<E>,
    sender: crate::impls::CustomActionSender<A, R>,
) -> (JoinHandle<()>, JoinHandle<()>)
where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    use futures_util::{SinkExt, StreamExt};
    use tokio_tungstenite::tungstenite::Message;

    let (mut sink, mut stream) = ws_stream.split();
    let (resp_sender, mut resp_receiver) = tokio::sync::mpsc::channel(1024);
    let sink_join = tokio::spawn(async move {
        loop {
            let s = tokio::select! {
                event = listener.recv() => {
                    if let Ok(event) = event {
                        serde_json::to_string(&event).unwrap()
                    }
                    else { panic!() }
                }
                resp = resp_receiver.recv() => { serde_json::to_string(&resp).unwrap() }
            };
            sink.send(Message::Text(s)).await.unwrap();
        }
    });
    let stream_join = tokio::spawn(async move {
        loop {
            if let Some(data) = stream.next().await {
                if let Ok(message) = data {
                    match serde_json::from_str(&message.to_string()) {
                        Ok(action) => {
                            sender
                                .send((action, crate::impls::CustomARSS::Mpsc(resp_sender.clone())))
                                .await
                                .unwrap();
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    });
    (sink_join, stream_join)
}

// #[cfg(all(feature = "websocket", feature = "impl"))]
// pub(crate) async fn sdk_websocket_loop(ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>) {}
