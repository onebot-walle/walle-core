use super::{AppOBC, BotMap, EchoMap};
use crate::{
    action::ActionType,
    comms::utils::AuthReqHeaderExt,
    config::{WebSocketClient, WebSocketServer},
    next::{ECAHtrait, EHACtrait, OneBotExt, Static},
    utils::{Echo, ProtocolItem},
    SelfId, WalleError, WalleResult,
};

use std::sync::Arc;

use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
use tokio::{net::TcpListener, sync::mpsc};
use tokio::{net::TcpStream, sync::oneshot};
use tokio_tungstenite::tungstenite::http::{header::USER_AGENT, Request};
use tokio_tungstenite::tungstenite::Message as WsMsg;
use tokio_tungstenite::WebSocketStream;
use tracing::{info, warn};

#[async_trait]
impl<E, A, R, OB> ECAHtrait<E, A, R, OB, Vec<WebSocketClient>> for AppOBC<A, R>
where
    E: ProtocolItem + SelfId + Clone,
    A: ProtocolItem + ActionType,
    R: ProtocolItem,
    OB: Static,
{
    async fn ecah_start<C0>(
        &self,
        ob: &Arc<OB>,
        config: Vec<WebSocketClient>,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        OB: EHACtrait<E, A, R, OB, C0> + OneBotExt,
    {
        let mut tasks = vec![];
        for wsc in config {
            info!(target: super::OBC, "Start try connect to {}", wsc.url);
            let ob = ob.clone();
            let echo_map = self.echos.clone();
            let bot_map = self.bots.clone();
            let mut signal_rx = ob.get_signal_rx().await?;
            tasks.push(tokio::spawn(async move {
                while signal_rx.try_recv().is_err() {
                    let ob = ob.clone();
                    let echo_map = echo_map.clone();
                    let bot_map = bot_map.clone();
                    let req = Request::builder()
                        .header(
                            USER_AGENT,
                            format!(
                                "OneBot/{} Walle-App/{}",
                                ob.get_onebot_version(),
                                crate::VERSION
                            ),
                        )
                        .header_auth_token(&wsc.access_token);
                    match crate::comms::ws_utils::try_connect(&wsc, req).await {
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
        Ok(tasks)
    }
    async fn handle_action(&self, id: &str, action: A, _ob: &OB) -> WalleResult<R> {
        match self.bots.get(id) {
            Some(action_tx) => {
                let (tx, rx) = oneshot::channel();
                let seq = self.next_seg();
                self.echos.insert(seq.clone(), tx);
                action_tx.send(seq.pack(action)).map_err(|e| {
                    warn!("send action error: {}", e);
                    WalleError::Other(e.to_string())
                })?;
                match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
                    Ok(Ok(res)) => Ok(res),
                    Ok(Err(e)) => {
                        warn!("resp recv error: {:?}", e);
                        Err(WalleError::Other(e.to_string()))
                    }
                    Err(_) => {
                        warn!("resp timeout");
                        Err(WalleError::Other("resp timeout".to_string()))
                    }
                }
            }
            None => {
                warn!("bot not found");
                return Err(WalleError::BotNotExist);
            }
        }
    }
}

#[async_trait]
impl<E, A, R, OB> ECAHtrait<E, A, R, OB, Vec<WebSocketServer>> for AppOBC<A, R>
where
    E: ProtocolItem + SelfId + Clone,
    A: ProtocolItem + ActionType,
    R: ProtocolItem,
    OB: Static,
{
    async fn ecah_start<C0>(
        &self,
        ob: &Arc<OB>,
        config: Vec<WebSocketServer>,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        OB: EHACtrait<E, A, R, OB, C0> + OneBotExt,
    {
        let mut tasks = vec![];
        for wss in config {
            let addr = std::net::SocketAddr::new(wss.host, wss.port);
            let tcp_listener = TcpListener::bind(&addr).await.map_err(WalleError::IO)?;
            info!(
                target: super::OBC,
                "Websocket server listening on ws://{}", addr
            );
            let ob = ob.clone();
            let mut signal_rx = ob.get_signal_rx().await?;
            let echo_map = self.echos.clone();
            let bot_map = self.bots.clone();
            tasks.push(tokio::spawn(async move {
                loop {
                    let ob = ob.clone();
                    let echo_map = echo_map.clone();
                    let bot_map = bot_map.clone();
                    tokio::select! {
                        _ = signal_rx.recv() => {
                            info!(target: super::OBC, "Stop listening on ws://{}", addr);
                            break;
                        }
                        Ok((stream, _)) = tcp_listener.accept() => {
                            if let Some(ws_stream) =
                                crate::comms::ws_utils::upgrade_websocket(&wss.access_token, stream)
                                    .await
                            {
                                let ob = ob.clone();
                                tokio::spawn(async move { ws_loop(ob, ws_stream, echo_map, bot_map).await });
                            }
                        }
                    }
                }
            }));
        }
        Ok(tasks)
    }
    async fn handle_action(&self, id: &str, action: A, _ob: &OB) -> WalleResult<R> {
        match self.bots.get(id) {
            Some(action_tx) => {
                let (tx, rx) = oneshot::channel();
                let seq = self.next_seg();
                self.echos.insert(seq.clone(), tx);
                action_tx.send(seq.pack(action)).map_err(|e| {
                    warn!(target: super::OBC, "send action error: {}", e);
                    WalleError::Other(e.to_string())
                })?;
                match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
                    Ok(Ok(res)) => Ok(res),
                    Ok(Err(e)) => {
                        warn!(target: super::OBC, "resp recv error: {:?}", e);
                        Err(WalleError::Other(e.to_string()))
                    }
                    Err(_) => {
                        warn!(target: super::OBC, "resp timeout");
                        Err(WalleError::Other("resp timeout".to_string()))
                    }
                }
            }
            None => {
                warn!(target: super::OBC, "bot not found");
                return Err(WalleError::BotNotExist);
            }
        }
    }
}

async fn ws_loop<E, A, R, OB, C>(
    ob: Arc<OB>,
    mut ws_stream: WebSocketStream<TcpStream>,
    echo_map: EchoMap<R>,
    bot_map: BotMap<A>,
) where
    E: ProtocolItem + SelfId + Clone,
    A: ProtocolItem + ActionType,
    R: ProtocolItem,
    OB: EHACtrait<E, A, R, OB, C> + OneBotExt + Static,
{
    let (action_tx, mut action_rx) = mpsc::unbounded_channel::<Echo<A>>();
    let mut signal_rx = ob.get_signal_rx().await.unwrap(); //todo
    loop {
        tokio::select! {
            _ = signal_rx.recv() => break,
            Some(action) = action_rx.recv() => {
                let content_type = action.inner.content_type();
                if ws_stream.send(action.to_ws_msg(content_type)).await.is_err() {
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
                        &action_tx
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

async fn ws_recv<E, A, R, OB, C>(
    msg: WsMsg,
    ob: &Arc<OB>,
    ws_stream: &mut WebSocketStream<TcpStream>,
    echo_map: &EchoMap<R>,
    bot_map: &BotMap<A>,
    action_tx: &mpsc::UnboundedSender<Echo<A>>,
) -> bool
where
    E: ProtocolItem + Clone + SelfId,
    R: ProtocolItem,
    OB: EHACtrait<E, A, R, OB, C> + Static,
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
                if bot_map.get(&self_id).is_none() {
                    bot_map.insert(self_id, action_tx.clone());
                }
                let ob = ob.clone();
                tokio::spawn(async move { ob.handle_event(event, &ob).await });
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
