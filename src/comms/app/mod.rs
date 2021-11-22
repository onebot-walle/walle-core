#[cfg(feature = "websocket")]
mod websocket;
#[cfg(feature = "websocket")]
mod websocket_rev;

#[cfg(feature = "websocket")]
pub(crate) use websocket::run as websocket_run;
#[cfg(feature = "websocket")]
pub(crate) use websocket_rev::run as websocket_rev_run;

#[cfg(feature = "websocket")]
use crate::app::CustomOneBot;
#[cfg(feature = "websocket")]
use std::sync::Arc;

#[cfg(feature = "websocket")]
async fn websocket_loop<E, A, R>(
    ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    ob: Arc<CustomOneBot<E, A, R>>,
) where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    use crate::event::BaseEvent;
    use crate::utils::Echo;
    use crate::ActionResp;
    use colored::*;
    use futures_util::{SinkExt, StreamExt};
    use serde::Deserialize;
    use tokio_tungstenite::tungstenite::Message;
    use tracing::{trace, warn};

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum ReceiveItem<E, R> {
        Event(BaseEvent<E>),
        Resp(Echo<ActionResp<R>>),
    }

    let (mut w, mut r) = ws_stream.split();
    let move_ob = ob.clone();

    let wj = tokio::spawn(async move {
        let mut receiver = move_ob.action_receiver.write().await;
        while let Some(action) = receiver.recv().await {
            let action = serde_json::to_string(&action).unwrap();
            if let Err(e) = w.send(Message::text(action)).await {
                let self_id = move_ob.self_id().await;
                warn!(target: "Walle-core", "[{}] ws disconnect with error {}", self_id.red(), e);
                return;
            }
        }
    });

    let rj = tokio::spawn(async move {
        while let Some(message) = r.next().await {
            let self_id = ob.self_id().await;
            match message {
                Ok(message) => {
                    match serde_json::from_str::<ReceiveItem<E, R>>(&message.to_string()) {
                        Ok(item) => match item {
                            ReceiveItem::Event(e) => {
                                trace!(target:"Walle-core","[{}] receive event {:?}", self_id.red(), e);
                                let handler = ob.event_handler.clone();
                                ob.set_id(&e.self_id).await;
                                tokio::spawn(async move { handler.handle(e).await });
                            }
                            ReceiveItem::Resp(r) => {
                                let (resp, echo) = r.unpack();
                                trace!(target:"Walle-core","[{}] receive action_resp {:?}", self_id.red(), resp);
                                if let Some((_, s)) = ob.echo_map.remove(&echo) {
                                    match s.send(resp) {
                                        _ => {}
                                    }
                                }
                            }
                        },
                        Err(_) => warn!(
                            target: "Walle-core",
                            "[{}] receive illegal event or resp {:?}",
                            self_id.red(),
                            message.to_string()
                        ),
                    }
                }
                Err(e) => {
                    warn!(target: "Walle-core", "[{}] ws disconnect with error {}", self_id.red(), e);
                    return;
                }
            }
        }
    });

    tokio::select! {
        _ = wj => {}
        _ = rj => {}
    }
}
