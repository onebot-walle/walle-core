use crate::{
    error::{WalleError, WalleResult},
    resp::{resp_error, Resp},
    util::{AuthReqHeaderExt, Echo, ProtocolItem, ValueMap},
    ActionHandler, EventHandler, OneBot,
};
use crate::{
    event::Event,
    obc::{
        ws_util::{try_connect, upgrade_websocket},
        ImplOBC,
    },
};
use futures_util::{SinkExt, StreamExt};
use std::{sync::Arc, time::Duration};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::http::{header::USER_AGENT, Request};
use tokio_tungstenite::tungstenite::Message as WsMsg;
use tokio_tungstenite::WebSocketStream;
use tracing::{info, trace, warn};

impl<E> ImplOBC<E>
where
    E: ProtocolItem + Clone,
{
    pub(crate) async fn ws<A, R, AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Vec<crate::config::WebSocketServer>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        A: ProtocolItem,
        R: ProtocolItem,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
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
            let mut signal_rx = ob.get_signal_rx()?;
            let event_rx = self.event_tx.subscribe();
            let hb_rx = self.hb_tx.subscribe();
            let ob = ob.clone();
            let implt = self.implt.clone();
            tasks.push(tokio::spawn(async move {
            loop { tokio::select! {
                    Ok((stream, addr)) = tcp_listener.accept() => {
                        if let Some(ws_stream) = upgrade_websocket(&access_token, stream).await {
                            info!(target: super::OBC, "New websocket connection from {}", addr);
                            tokio::spawn(ws_loop(
                                ob.clone(),
                                implt.clone(),
                                event_rx.resubscribe(),
                                hb_rx.resubscribe(),
                                ws_stream,
                            ));
                        }
                    }
                    _ = signal_rx.recv() => break,
                }}
            }));
        }
        Ok(())
    }

    pub(crate) async fn wsr<A, R, AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Vec<crate::config::WebSocketClient>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        A: ProtocolItem,
        R: ProtocolItem,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        for wsr in config {
            let r#impl = self.implt.clone();
            let event_rx = self.event_tx.subscribe();
            let hb_rx = self.hb_tx.subscribe();
            let mut signal_rx = ob.get_signal_rx()?;
            let ob = ob.clone();
            let implt = self.implt.clone();
            tasks.push(tokio::spawn(async move {
                info!(target: super::OBC, "Start try connect to {}", wsr.url);
                while signal_rx.try_recv().is_err() {
                    let req = Request::builder()
                        .header(
                            USER_AGENT,
                            format!("OneBot/{} Walle/{}", 12, crate::VERSION),
                        )
                        .header("X-OneBot-Version", 12.to_string())
                        .header("X-Impl", r#impl.clone())
                        .header("X-Client-Role", "Universal".to_string()) // for v11
                        .header_auth_token(&wsr.access_token);
                    match try_connect(&wsr, req).await {
                        Some(ws_stream) => {
                            ws_loop(
                                ob.clone(),
                                implt.clone(),
                                event_rx.resubscribe(),
                                hb_rx.resubscribe(),
                                ws_stream,
                            )
                            .await;
                            warn!(target: super::OBC, "Disconnected from {}", wsr.url);
                        }
                        None => {
                            tokio::time::sleep(std::time::Duration::from_secs(
                                wsr.reconnect_interval as u64,
                            ))
                            .await;
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
    implt: String,
    mut event_rx: broadcast::Receiver<E>,
    mut hb_rx: broadcast::Receiver<Event>,
    mut ws_stream: WebSocketStream<TcpStream>,
) where
    E: ProtocolItem + Clone,
    A: ProtocolItem,
    R: ProtocolItem,
    AH: ActionHandler<E, A, R> + Send + Sync + 'static,
    EH: EventHandler<E, A, R> + Send + Sync + 'static,
{
    let (json_resp_tx, mut json_resp_rx) = tokio::sync::mpsc::unbounded_channel();
    let (rmp_resp_tx, mut rmp_resp_rx) = tokio::sync::mpsc::unbounded_channel();
    let mut signal_rx = ob.get_signal_rx().unwrap(); //todo
    let status = ob.action_handler.get_status().await;
    let event = Event {
        id: "".to_string(),
        implt: implt.to_string(),
        time: crate::util::timestamp_nano_f64(),
        ty: "meta".to_string(),
        detail_type: "status_update".to_string(),
        sub_type: "".to_string(),
        extra: status.into(),
    };
    ws_stream.send(WsMsg::Text(event.json_encode())).await.ok();
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
                    Ok(ws_msg) => if ws_recv(
                            ws_msg,
                            &ob,
                            &mut ws_stream,
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

pub(crate) async fn ws_recv<E, A, R, AH, EH>(
    ws_msg: WsMsg,
    ob: &Arc<OneBot<AH, EH>>,
    ws_stream: &mut WebSocketStream<TcpStream>,
    json_resp_sender: &tokio::sync::mpsc::UnboundedSender<Echo<R>>,
    rmp_resp_sender: &tokio::sync::mpsc::UnboundedSender<Echo<R>>,
) -> bool
where
    E: ProtocolItem,
    A: ProtocolItem,
    R: ProtocolItem,
    AH: ActionHandler<E, A, R> + Send + Sync + 'static,
    EH: EventHandler<E, A, R> + Send + Sync + 'static,
{
    let err_handle = |a: Echo<ValueMap>, msg: String| -> Echo<Resp> {
        let (_, echo_s) = a.unpack();
        warn!(target: crate::WALLE_CORE, "action warn: {}", msg);
        if msg.starts_with("missing field") {
            echo_s.pack(Resp::from(resp_error::bad_segment_data(msg)))
        } else {
            echo_s.pack(resp_error::unsupported_action(msg).into())
        }
    };

    match ws_msg {
        WsMsg::Text(text) => match serde_json::from_str::<'_, Echo<A>>(&text) {
            Ok(action) => {
                let (action, echos) = action.unpack();
                let tx = json_resp_sender.clone();
                let ob = ob.clone();
                tokio::spawn(async move {
                    tokio::time::timeout(Duration::from_secs(10), async move {
                        match ob.handle_action(action).await {
                            Ok(r) => {
                                tx.send(echos.pack(r)).ok();
                            }
                            Err(e) => warn!(target: super::OBC, "handle action error: {}", e),
                        }
                    })
                    .await
                });
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
        WsMsg::Binary(v) => match rmp_serde::from_read::<_, Echo<A>>(v.as_slice()) {
            Ok(action) => {
                let (action, echos) = action.unpack();
                let tx = rmp_resp_sender.clone();
                let ob = ob.clone();
                tokio::spawn(async move {
                    tokio::time::timeout(Duration::from_secs(10), async move {
                        match ob.handle_action(action).await {
                            Ok(r) => {
                                tx.send(echos.pack(r)).ok();
                            }
                            Err(e) => warn!(target: super::OBC, "handle action error: {}", e),
                        }
                    })
                    .await
                });
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
