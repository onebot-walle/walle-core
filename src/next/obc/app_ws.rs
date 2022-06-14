use super::{AppOBC, BotMap, EchoMap};
use crate::{
    action::ActionType,
    config::WebSocketClient,
    next::{ActionContext, ECAHtrait, OneBot, Static},
    utils::{Echo, EchoS, ProtocolItem},
    SelfId, WalleResult,
};

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::Message as WsMsg;
use tokio_tungstenite::WebSocketStream;
use tracing::warn;

#[async_trait]
impl<E, A, R, EHAC, const V: u8>
    ECAHtrait<E, Echo<A>, Echo<R>, OneBot<E, A, R, Self, EHAC, V>, Vec<WebSocketClient>>
    for AppOBC<A, R>
where
    E: ProtocolItem + SelfId + Clone,
    A: ProtocolItem + ActionType,
    R: ProtocolItem,
    EHAC: Static,
{
    async fn start(
        &self,
        ob: &Arc<OneBot<E, A, R, Self, EHAC, V>>,
        config: Vec<WebSocketClient>,
    ) -> WalleResult<(Vec<JoinHandle<()>>, broadcast::Receiver<E>)> {
        todo!()
    }
    async fn handle(
        &self,
        action_context: ActionContext<Echo<A>, Echo<R>>,
        ob: &OneBot<E, A, R, Self, EHAC, V>,
    ) {
        match self.bots.read().await.get(&action_context.0) {
            Some(tx) => {
                let echo_s = action_context.1.get_echo();
                self.echos.lock().await.insert(echo_s, action_context.2);
                tx.send(action_context.1).ok();
            }
            None => {
                warn!("bot not found");
                return;
            }
        }
    }
}

async fn ws_loop<E, A, R>(
    echo_map: EchoMap<R>,
    mut ws_stream: WebSocketStream<TcpStream>,
    mut signal_rx: broadcast::Receiver<()>,
    event_tx: broadcast::Sender<E>,
    mut action_rx: mpsc::UnboundedReceiver<Echo<A>>,
) where
    E: ProtocolItem + SelfId + Clone,
    A: ProtocolItem + ActionType,
    R: ProtocolItem,
{
    // let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Echo<A>>();
    loop {
        tokio::select! {
            _ = signal_rx.recv() => break,
            Some(action) = action_rx.recv() => {
                let content_type = action.inner.content_type();
                if ws_stream.send(action.to_ws_msg(content_type)).await.is_err() {
                    break;
                }
                // remove todo
            },
            Some(msg) = ws_stream.next() => {
                match msg {
                    Ok(msg) => if ws_recv(
                        msg,
                        &mut ws_stream,
                        &echo_map,
                        &event_tx,
                    ).await {
                        break;
                    },
                    Err(_) => {
                        break;
                    }
                }
            }
        }
    }
    ws_stream.send(WsMsg::Close(None)).await.ok();
}

async fn ws_recv<E, R>(
    msg: WsMsg,
    ws_stream: &mut WebSocketStream<TcpStream>,
    echo_map: &EchoMap<R>,
    event_tx: &broadcast::Sender<E>,
) -> bool
where
    E: ProtocolItem + Clone + SelfId,
    R: ProtocolItem,
{
    #[derive(Debug, Deserialize, Serialize)]
    #[serde(untagged)]
    enum ReceiveItem<E, R> {
        Event(E),
        Resp(Echo<R>),
    }

    let handle_ok = |item: Result<ReceiveItem<E, R>, String>| async move {
        match item {
            Ok(ReceiveItem::Event(event)) => {
                event_tx.send(event).ok();
            }
            Ok(ReceiveItem::Resp(resp)) => {
                let echos = resp.get_echo();
                if let Some(rx) = echo_map.lock().await.remove(&echos) {
                    rx.send(resp).ok();
                }
            }
            Err(s) => warn!(target: crate::WALLE_CORE, "serde failed: {}", s),
        }
    };

    match msg {
        WsMsg::Text(text) => handle_ok(ProtocolItem::json_decode(&text)).await,
        WsMsg::Binary(bin) => handle_ok(ProtocolItem::rmp_decode(&bin)).await,
        WsMsg::Ping(b) => {
            if ws_stream.send(WsMsg::Pong(b)).await.is_err() {
                return true;
            }
        }
        WsMsg::Pong(_) => {}
        WsMsg::Close(_) => return true,
        WsMsg::Frame(_) => unreachable!(),
    }
    false
}
