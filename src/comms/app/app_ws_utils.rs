use futures_util::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot, RwLock},
};
use tokio_tungstenite::{tungstenite::Message as WsMsg, WebSocketStream};

use crate::{app::CustomOneBot, ActionResp, BaseEvent, Echo, EchoS};

impl<E, A, R> CustomOneBot<E, A, R>
where
    E: Clone + DeserializeOwned + Send + 'static + Debug,
    A: Clone + Serialize + Send + 'static + Debug,
    R: Clone + DeserializeOwned + Send + 'static + Debug,
{
    async fn ws_loop(self: &Arc<Self>, mut ws_stream: WebSocketStream<TcpStream>) {
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        let mut bot_ids: Vec<String> = vec![];
        let echo_map = RwLock::default();
        loop {
            tokio::select! {
                echo_action = action_rx.recv() => {
                    if let Some(echo_action) = echo_action {
                        let action = serde_json::to_string(&echo_action).unwrap();
                        ws_stream.send(WsMsg::Text(action)).await;
                    }
                },
                msg = ws_stream.next() => {
                    if let Some(msg) = msg {
                        match msg {
                            Ok(msg) => {
                                self.ws_recv(msg, &mut bot_ids,&action_tx, &echo_map).await;
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }
        }
    }

    async fn ws_recv(
        self: &Arc<Self>,
        ws_msg: WsMsg,
        bot_ids: &mut Vec<String>,
        action_rx: &mpsc::UnboundedSender<Echo<A>>,
        echo_map: &RwLock<HashMap<EchoS, oneshot::Sender<ActionResp<R>>>>,
    ) {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum ReceiveItem<E, R> {
            Event(BaseEvent<E>),
            Resp(Echo<ActionResp<R>>),
        }

        if let WsMsg::Text(text) = ws_msg {
            let item: ReceiveItem<E, R> = serde_json::from_str(&text).unwrap();
            match item {
                ReceiveItem::Event(event) => {
                    let ob = self.clone();
                    tokio::spawn(async move { ob.event_handler.handle(event).await });
                }
                ReceiveItem::Resp(resp) => {
                    let (resp, echos) = resp.unpack();
                    if let Some(rx) = echo_map.write().await.remove(&echos) {
                        rx.send(resp).unwrap();
                    }
                }
            }
        }
    }
}
