use super::ImplOBC;
use crate::{
    comms::utils::AuthReqHeaderExt,
    comms::ws_utils::upgrade_websocket,
    error::{WalleError, WalleResult},
    next::{ActionContext, EHACtrait, OneBot, Static},
    resp::error_builder,
    utils::{Echo, ExtendedMap, ProtocolItem},
    Resps, StandardEvent,
};
use async_trait::async_trait;
use futures_util::{SinkExt, StreamExt};
use std::{fmt::Debug, sync::Arc};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::http::{header::USER_AGENT, Request};
use tokio_tungstenite::tungstenite::Message as WsMsg;
use tokio_tungstenite::WebSocketStream;
use tracing::{info, trace, warn};

impl<E> ImplOBC<E> {
    pub(crate) async fn ws<A, R, ECAH, const V: u8>(
        &self,
        ob: &Arc<OneBot<E, A, R, ECAH, Self, V>>,
        config: Vec<crate::config::WebSocketServer>,
        action_tx: mpsc::UnboundedSender<ActionContext<A, R>>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        E: ProtocolItem + Static + Clone,
        A: ProtocolItem + Static,
        R: ProtocolItem + Static + Debug,
        ECAH: Static,
    {
        for wss in config {
            let addr = std::net::SocketAddr::new(wss.host, wss.port);
            let tcp_listener = tokio::net::TcpListener::bind(&addr)
                .await
                .map_err(WalleError::from)?;
            info!(
                target: super::OBC,
                "Websocket server listening on ws://{}", addr
            );
            let access_token = wss.access_token.clone();
            let signal_rx = ob.get_signal_rx().await.unwrap();
            let obc = self.clone();
            let action_tx = action_tx.clone();
            tasks.push(tokio::spawn(async move {
                loop {
                    let access_token = access_token.clone();
                    let mut signal_rx = signal_rx.resubscribe();
                    let obc = obc.clone();
                    let action_tx = action_tx.clone();
                    tokio::select! {
                        Ok((stream, addr)) = tcp_listener.accept() => {
                            if let Some(ws_stream) = upgrade_websocket(&access_token, stream).await {
                                info!(target: super::OBC, "New websocket connection from {}", addr);
                                tokio::spawn(async move {
                                    ws_loop(
                                        obc.self_id, 
                                        obc.event_tx.subscribe(), 
                                        action_tx.clone(), 
                                        obc.hb_tx.subscribe(), 
                                        ws_stream, 
                                        signal_rx.resubscribe()
                                    ).await;
                                });
                            }
                        }
                        _ = signal_rx.recv() => break,
                    }
                }
            }));
        }
        Ok(())
    }

    pub(crate) async fn wsr<A, R, ECAH, const V: u8>(
        &self,
        ob: &Arc<OneBot<E, A, R, ECAH, Self, V>>,
        config: Vec<crate::config::WebSocketClient>,
        action_tx: mpsc::UnboundedSender<ActionContext<A, R>>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        E: ProtocolItem + Static + Clone,
        A: ProtocolItem + Static,
        R: ProtocolItem + Static + Debug,
        ECAH: Static,
    {
        for wsr in config {
            let obc = self.clone();
            let action_tx = action_tx.clone();
            let mut signal_rx = ob.get_signal_rx().await.unwrap();
            tasks.push(tokio::spawn(async move {
                info!(
                    target: crate::WALLE_CORE,
                    "Start try connect to {}", wsr.url
                );
                loop {
                    let req = Request::builder()
                        .header(
                            USER_AGENT,
                            format!("OneBot/{} ({}) Walle/{}", V, obc.platform, crate::VERSION),
                        )
                        .header("X-OneBot-Version", V.to_string())
                        .header("X-Platform", obc.platform.clone())
                        .header("X-Impl", obc.r#impl.clone())
                        .header("X-Self-ID", obc.self_id.read().await.as_str())
                        .header("X-Client-Role", "Universal".to_string()) // for v11
                        .header_auth_token(&wsr.access_token);
                    tokio::select! {
                        s = crate::comms::ws_utils::try_connect(&wsr, req) => {
                            match s {
                                Some(ws_stream) => {
                                    ws_loop(
                                        obc.self_id.clone(), 
                                        obc.event_tx.subscribe(), 
                                        action_tx.clone(), 
                                        obc.hb_tx.subscribe(), 
                                        ws_stream, 
                                        signal_rx.resubscribe()
                                    ).await;
                                    warn!(target: crate::WALLE_CORE, "Disconnected from {}", wsr.url);
                                }
                                None => {
                                    tokio::time::sleep(std::time::Duration::from_secs(
                                        wsr.reconnect_interval as u64,
                                    ))
                                    .await;
                                }
                            }
                        }
                        _ = signal_rx.recv() => break,
                    }
                }
            }));
        }
        Ok(())
    }
}

#[async_trait]
impl<E, A, R, ECAH, const V: u8>
    EHACtrait<E, A, R, OneBot<E, A, R, ECAH, Self, V>, Vec<crate::config::WebSocketServer>>
    for ImplOBC<E>
where
    E: ProtocolItem + Static + Clone,
    A: ProtocolItem + Static,
    R: ProtocolItem + Static + Debug,
    ECAH: Static,
{
    async fn start(
        &self,
        ob: &Arc<OneBot<E, A, R, ECAH, Self, V>>,
        config: Vec<crate::config::WebSocketServer>,
    ) -> WalleResult<(
        Vec<JoinHandle<()>>,
        mpsc::UnboundedReceiver<ActionContext<A, R>>,
    )> {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<ActionContext<A, R>>();
        let mut tasks = vec![];
        self.ws(ob, config, action_tx, &mut tasks).await?;
        Ok((tasks, action_rx))
    }
    async fn handle(&self, event: E, _ob: &OneBot<E, A, R, ECAH, Self, V>) {
        self.event_tx.send(event).ok();
    }
}

#[async_trait]
impl<E, A, R, ECAH, const V: u8>
    EHACtrait<E, A, R, OneBot<E, A, R, ECAH, Self, V>, Vec<crate::config::WebSocketClient>>
    for ImplOBC<E>
where
    E: ProtocolItem + Static + Clone,
    A: ProtocolItem + Static,
    R: ProtocolItem + Static + Debug,
    ECAH: Static,
{
    async fn start(
        &self,
        ob: &Arc<OneBot<E, A, R, ECAH, Self, V>>,
        config: Vec<crate::config::WebSocketClient>,
    ) -> WalleResult<(
        Vec<JoinHandle<()>>,
        mpsc::UnboundedReceiver<ActionContext<A, R>>,
    )> {
        let (action_tx, action_rx) = mpsc::unbounded_channel::<ActionContext<A, R>>();
        let mut tasks = vec![];
        self.wsr(ob, config, action_tx, &mut tasks).await?;
        Ok((tasks, action_rx))
    }
    async fn handle(&self, event: E, _ob: &OneBot<E, A, R, ECAH, Self, V>) {
        self.event_tx.send(event).ok();
    }
}


async fn ws_loop<E, A, R>(
    id: Arc<tokio::sync::RwLock<String>>,
    mut event_rx: broadcast::Receiver<E>,
    mut action_tx: mpsc::UnboundedSender<ActionContext<A, R>>,
    mut hb_rx: broadcast::Receiver<StandardEvent>,
    mut ws_stream: WebSocketStream<TcpStream>,
    mut signal_rx: broadcast::Receiver<()>,
) where
    E: ProtocolItem + Clone,
    A: ProtocolItem,
    R: ProtocolItem + Debug,
{
    // action response channel
    let (json_resp_tx, mut json_resp_rx) = tokio::sync::mpsc::unbounded_channel();
    let (rmp_resp_tx, mut rmp_resp_rx) = tokio::sync::mpsc::unbounded_channel();
    loop {
        tokio::select! {
            _ = signal_rx.recv() => break,
            event = event_rx.recv() => {
                match event {
                    Ok(event) => {
                        // event will always send as json
                        let event = event.json_encode();
                        trace!(target: crate::WALLE_CORE, "ws send: {}", event);
                        if ws_stream.send(WsMsg::Text(event)).await.is_err() {
                            // send failed, break loop and close connection
                            break;
                        }
                    }
                    Err(_) => {
                        // channel all sender are dropped or channel is fulled will break loop and close connection
                        break;
                    }
                }
            },
            hb = hb_rx.recv() => {
                match hb {
                    Ok(hb) => {
                        let hb = hb.json_encode();
                        trace!(target: crate::WALLE_CORE, "ws send: {}", hb);
                        if ws_stream.send(WsMsg::Text(hb)).await.is_err() {
                            break;
                        }
                    }
                    Err(_) => {
                        break;
                    }
                }
            }
            Some(ws_msg) = ws_stream.next() => {
                trace!(target: crate::WALLE_CORE, "ws recv: {:?}", ws_msg);
                match ws_msg {
                    // handle action request
                    Ok(ws_msg) => if ws_recv::<E, _, _>(
                            ws_msg,
                            &id.read().await,
                            &mut ws_stream,
                            &mut action_tx,
                            &json_resp_tx,
                            &rmp_resp_tx
                        ).await { break },
                    Err(_) => break,
                }

            },
            Some(resp) = json_resp_rx.recv() => {
                trace!(target: crate::WALLE_CORE, "ws send json: {:?}", resp);
                // send action response
                if ws_stream.send(WsMsg::Text(resp.json_encode())).await.is_err() {
                    break;
                }

            },
            Some(resp) = rmp_resp_rx.recv() => {
                trace!(target: crate::WALLE_CORE, "ws send rmp: {:?}", resp);
                // send action response
                if ws_stream.send(WsMsg::Binary(resp.rmp_encode())).await.is_err() {
                    break;
                }
            }
        }
    }
    ws_stream.send(WsMsg::Close(None)).await.ok();
}

pub(crate) async fn ws_recv<E, A, R>(
    ws_msg: WsMsg,
    id: &str,
    ws_stream: &mut WebSocketStream<TcpStream>,
    action_tx: &mut mpsc::UnboundedSender<ActionContext<A, R>>,
    json_resp_sender: &tokio::sync::mpsc::UnboundedSender<R>,
    rmp_resp_sender: &tokio::sync::mpsc::UnboundedSender<R>,
) -> bool
where
    E: ProtocolItem,
    A: ProtocolItem,
    R: ProtocolItem,
{
    let err_handle = |a: Echo<ExtendedMap>, msg: String| -> Echo<Resps<E>> {
        let (_, echo_s) = a.unpack();
        warn!(target: crate::WALLE_CORE, "action warn: {}", msg);
        if msg.starts_with("missing field") {
            echo_s.pack(crate::Resps::empty_fail(10006, msg))
        } else {
            echo_s.pack(error_builder::unsupported_action().into())
        }
    };

    match ws_msg {
        WsMsg::Text(text) => match serde_json::from_str(&text) {
            Ok(echo_action) => {
                action_tx
                    .send((id.to_string(), echo_action, json_resp_sender.clone()))
                    .ok();
                //todo
            }
            Err(msg) => match serde_json::from_str(&text) {
                Ok(a) => {
                    let resp = serde_json::to_string(&err_handle(a, msg.to_string())).unwrap();
                    if ws_stream.send(WsMsg::Text(resp)).await.is_err() {
                        return true;
                    }
                }
                Err(_) => {
                    tracing::warn!(
                        target: crate::WALLE_CORE,
                        "json deserialize failed: {:?}",
                        text
                    )
                }
            },
        },
        WsMsg::Binary(v) => match rmp_serde::from_read(v.as_slice()) {
            Ok(echo_action) => {
                action_tx
                    .send((id.to_string(), echo_action, rmp_resp_sender.clone()))
                    .ok(); //todo
            }
            Err(msg) => match rmp_serde::from_read(v.as_slice()) {
                Ok(a) => {
                    let resp = rmp_serde::to_vec(&err_handle(a, msg.to_string())).unwrap();
                    if ws_stream.send(WsMsg::Binary(resp)).await.is_err() {
                        return true;
                    }
                }
                Err(_) => {
                    tracing::warn!(target: crate::WALLE_CORE, "rmp deserialize failed: {:?}", v)
                }
            },
        },
        WsMsg::Ping(b) => {
            if ws_stream.send(WsMsg::Pong(b)).await.is_err() {
                return true;
            }
        }
        WsMsg::Close(_) => return true,
        _ => {}
    }
    false
}
