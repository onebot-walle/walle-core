use futures_util::{SinkExt, StreamExt};
use tokio::{net::TcpStream, task::JoinHandle};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

pub enum ContentTpye {
    Json,
    MsgPack,
}

impl ContentTpye {
    pub fn new(s: &str) -> Option<Self> {
        match s {
            "application/json" => Some(Self::Json),
            "application/msgpack" => Some(Self::MsgPack),
            _ => None,
        }
    }
}

pub async fn web_socket_loop(
    ws_stream: WebSocketStream<TcpStream>,
    mut listener: crate::EventListner,
    sender: crate::ActionSender,
) -> (JoinHandle<()>, JoinHandle<()>) {
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
                            sender.send((action, crate::ARSS::Mpsc(resp_sender.clone())));
                        }
                        Err(_) => {}
                    }
                }
            }
        }
    });
    (sink_join, stream_join)
}
