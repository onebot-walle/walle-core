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
    mut action_listener: crate::app::CustomActionListenr<A, R>,
) where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    use crate::event::BaseEvent;
    use crate::utils::{Echo, EchoS};
    use colored::*;
    use futures_util::{SinkExt, StreamExt};
    use serde::Deserialize;
    use tokio_tungstenite::tungstenite::Message;
    use tracing::error;

    #[derive(Debug, Deserialize)]
    #[serde(untagged)]
    enum ReceiveItem<E, R> {
        Event(BaseEvent<E>),
        Resp(Echo<R>),
    }

    let mut waitting_group = std::collections::HashMap::new();
    let mut self_id = "".to_owned();

    loop {
        tokio::select! {
        action_result = action_listener.recv() =>
        {
            if let Ok((action, sender)) = action_result {
                let echo = EchoS::new(&self_id);
                waitting_group.insert(echo.clone(), sender);
                let action = echo.pack(action);
                let action = serde_json::to_string(&action).unwrap();
                if let Err(e) = ws_stream.send(Message::text(action)).await {
                    error!(target: "Walle-core", "[{}] ws disconnect with error {}", self_id.red(), e);
                    return;
                }
            }
        }
        option_data = ws_stream.next() =>
        {
            if let Some(data) = option_data {
                match data {
                    Ok(message) => {
                        match serde_json::from_str::<ReceiveItem<E, R>>(&message.to_string()) {
                            Ok(item) => match item {
                                ReceiveItem::Event(e) => {
                                    let handler = event_handler.clone();
                                    self_id = e.self_id.clone();
                                    tokio::spawn(async move { handler.handle(e).await });
                                }
                                ReceiveItem::Resp(r) => {
                                    let (resp, echo) = r.unpack();
                                    if let Some(s) = waitting_group.remove(&echo) {
                                        match s.send(resp).await {
                                            _ => {}
                                        }
                                    }
                                }
                            },
                            Err(_) => error!(
                                target: "Walle-core",
                                "[{}] receive illegal event or resp {:?}",
                                self_id.red(),
                                message.to_string()
                            ),
                        }
                    }
                    Err(e) => {
                        error!(target: "Walle-core", "[{}] ws disconnect with error {}", self_id.red(), e);
                        return;
                    }
                }
            }
        }
        }
    }
}
