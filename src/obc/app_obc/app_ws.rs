use crate::{
    config::{WebSocketClient, WebSocketServer},
    error::{WalleError, WalleResult},
    util::{AuthReqHeaderExt, Echo, ProtocolItem, SelfId},
    ActionHandler, EventHandler, OneBot,
};
use crate::{
    obc::{
        ws_util::{try_connect, upgrade_websocket},
        AppOBC, BotMap, BotMapExt, EchoMap,
    },
    util::ContentType,
};

use std::{collections::HashSet, sync::Arc};

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio::task::JoinHandle;
use tokio::{net::TcpListener, sync::mpsc};
use tokio_tungstenite::tungstenite::http::{header::USER_AGENT, Request};
use tokio_tungstenite::tungstenite::Message as WsMsg;
use tokio_tungstenite::WebSocketStream;
use tracing::{info, warn};

impl<A, R> AppOBC<A, R>
where
    A: ProtocolItem,
    R: ProtocolItem,
{
    pub(crate) async fn ws<E, AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Vec<WebSocketClient>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        E: ProtocolItem + SelfId + Clone,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        for wsc in config {
            info!(target: super::OBC, "Start try connect to {}", wsc.url);
            let ob = ob.clone();
            let echo_map = self.echos.clone();
            let bot_map = self.bots.clone();
            let mut signal_rx = ob.get_signal_rx()?;
            tasks.push(tokio::spawn(async move {
                while signal_rx.try_recv().is_err() {
                    let ob = ob.clone();
                    let echo_map = echo_map.clone();
                    let bot_map = bot_map.clone();
                    let req = Request::builder()
                        .header(
                            USER_AGENT,
                            format!("OneBot/12 Walle-App/{}", crate::VERSION),
                        )
                        .header_auth_token(&wsc.access_token);
                    match try_connect(&wsc, req).await {
                        Some(ws_stream) => {
                            ws_loop(ob, ws_stream, echo_map, bot_map).await;
                            warn!(target: crate::WALLE_CORE, "Disconnected from {}", wsc.url);
                        }
                        None => {
                            tokio::time::sleep(std::time::Duration::from_secs(
                                wsc.reconnect_interval as u64,
                            ))
                            .await;
                        }
                    }
                }
            }));
        }
        Ok(())
    }
    pub(crate) async fn wsr<E, AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Vec<WebSocketServer>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        E: ProtocolItem + SelfId + Clone,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        for wss in config {
            let addr = std::net::SocketAddr::new(wss.host, wss.port);
            let tcp_listener = TcpListener::bind(&addr).await.map_err(WalleError::IO)?;
            info!(
                target: super::OBC,
                "Websocket server listening on ws://{}", addr
            );
            let ob = ob.clone();
            let mut signal_rx = ob.get_signal_rx()?;
            let echo_map = self.echos.clone();
            let bot_map = self.bots.clone();
            tasks.push(tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = signal_rx.recv() => {
                            info!(target: super::OBC, "Stop listening on ws://{}", addr);
                            break;
                        }
                        Ok((stream, _)) = tcp_listener.accept() => {
                            if let Some(ws_stream) =
                                upgrade_websocket(&wss.access_token, stream)
                                    .await
                            {
                                let ob = ob.clone();
                                tokio::spawn(ws_loop(ob.clone(), ws_stream, echo_map.clone(), bot_map.clone()));
                            }
                        }
                    }
                }
            }));
        }
        Ok(())
    }
}

async fn ws_loop<E, A, R, AH, EH>(
    ob: Arc<OneBot<AH, EH>>,
    mut ws_stream: WebSocketStream<TcpStream>,
    echo_map: EchoMap<R>,
    bot_map: BotMap<A>,
) where
    E: ProtocolItem + SelfId + Clone,
    A: ProtocolItem,
    R: ProtocolItem,
    AH: ActionHandler<E, A, R> + Send + Sync + 'static,
    EH: EventHandler<E, A, R> + Send + Sync + 'static,
{
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Echo<A>>();
    let mut signal_rx = ob.get_signal_rx().unwrap(); //todo
    let mut bot_set = HashSet::default();
    loop {
        tokio::select! {
            _ = signal_rx.recv() => break,
            Some(action) = action_rx.recv() => {
                if ws_stream.send(action.to_ws_msg(&ContentType::Json)).await.is_err() { //todo
                    break;
                }
            },
            Some(msg) = ws_stream.next() => {
                match msg {
                    Ok(msg) => if ws_recv(
                        msg,
                        &ob,
                        &mut ws_stream,
                        &echo_map,
                        &bot_map,
                        &action_tx,
                        &mut bot_set,
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
    for bot in bot_set {
        bot_map.remove_bot(&bot, &action_tx);
    }
}

async fn ws_recv<E, A, R, AH, EH>(
    msg: WsMsg,
    ob: &Arc<OneBot<AH, EH>>,
    ws_stream: &mut WebSocketStream<TcpStream>,
    echo_map: &EchoMap<R>,
    bot_map: &BotMap<A>,
    action_tx: &mpsc::UnboundedSender<Echo<A>>,
    bot_set: &mut HashSet<String>,
) -> bool
where
    E: ProtocolItem + Clone + SelfId,
    A: ProtocolItem,
    R: ProtocolItem,
    AH: ActionHandler<E, A, R> + Send + Sync + 'static,
    EH: EventHandler<E, A, R> + Send + Sync + 'static,
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
                let self_id = event.self_id();
                bot_map.ensure_bot(&self_id, action_tx);
                bot_set.insert(self_id);
                let ob = ob.clone();
                tokio::spawn(async move { ob.handle_event(event).await });
            }
            Ok(ReceiveItem::Resp(resp)) => {
                let (r, echos) = resp.unpack();
                if let Some((_, tx)) = echo_map.remove(&echos) {
                    tx.send(r).ok();
                }
            }
            Err(s) => warn!(target: super::OBC, "serde failed: {}", s),
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
