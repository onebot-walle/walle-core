use crate::ExtendedMap;
use crate::{impls::CustomOneBot, Echo, WalleError, WalleLogExt, WalleResult};
use colored::*;
use futures_util::{SinkExt, StreamExt};
use serde::Deserialize;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::http::{header::USER_AGENT, Request};
use tokio_tungstenite::{tungstenite::Message as WsMsg, WebSocketStream};
use tracing::{debug, info, warn};

impl<E, A, R, const V: u8> CustomOneBot<E, A, R, V>
where
    E: Clone + Serialize + Send + 'static + Debug,
    A: DeserializeOwned + Send + 'static + Debug,
    R: Serialize + Send + 'static + Debug,
{
    pub(crate) async fn ws_loop(
        self: &Arc<Self>,
        mut ws_stream: WebSocketStream<TcpStream>,
    ) -> WalleResult<()> {
        self.ws_hooks.on_connect(self).await;
        let mut listener = self.broadcaster.subscribe();
        let (resp_tx, mut resp_rx) = tokio::sync::mpsc::unbounded_channel();
        loop {
            tokio::select! {
                event = listener.recv() => {
                    match event {
                        Ok(event) => {
                            let event = serde_json::to_string(&event).unwrap();
                            debug!(target: "Walle-core", "ws send: {}", event);
                            if ws_stream.send(WsMsg::Text(event)).await.is_err() {
                                break;
                            }
                        }
                        Err(_) => {
                            break;
                        }
                    }
                },
                ws_msg = ws_stream.next() => {
                    if let Some(ws_msg) = ws_msg {
                        debug!(target: "Walle-core", "ws recv: {:?}", ws_msg);
                        match ws_msg {
                            Ok(ws_msg) => if self.ws_recv(ws_msg, &resp_tx).await { break },
                            Err(_) => break,
                        }
                    }
                },
                resp = resp_rx.recv() => {
                    if let Some(resp) = resp {
                        debug!(target: "Walle-core", "ws send: {:?}", resp);
                        if ws_stream.send(resp).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
        self.ws_hooks.on_disconnect(self).await;
        Ok(())
    }

    pub(crate) async fn ws_recv(
        self: &Arc<Self>,
        ws_msg: WsMsg,
        resp_sender: &tokio::sync::mpsc::UnboundedSender<WsMsg>,
    ) -> bool {
        #[derive(Deserialize)]
        struct UnknownAction {
            action: String,
            #[allow(dead_code)]
            patams: ExtendedMap,
        }

        let ok_handle = |echo_action: Echo<A>, binary: bool| {
            let (action, echo_s) = echo_action.unpack();
            let sender = resp_sender.clone();
            let ob = self.clone();
            tokio::spawn(async move {
                let r = ob.action_handler.handle(action, &ob).await;
                let echo = echo_s.pack(r);
                if binary {
                    let resp = rmp_serde::to_vec(&echo).unwrap();
                    sender.send(WsMsg::Binary(resp)).unwrap();
                } else {
                    let resp = serde_json::to_string(&echo).unwrap();
                    sender.send(WsMsg::Text(resp)).unwrap();
                }
            });
        };
        let err_handle = |a: Echo<UnknownAction>| -> Echo<crate::Resps> {
            let (action, echo_s) = a.unpack();
            tracing::warn!(target: "Walle-core", "unsupported action: {}", action.action);
            echo_s.pack(crate::Resps::unsupported_action())
        };

        match ws_msg {
            WsMsg::Text(text) => match serde_json::from_str(&text) {
                Ok(echo_action) => {
                    ok_handle(echo_action, false);
                }
                Err(_) => match serde_json::from_str(&text) {
                    Ok(a) => {
                        let resp = serde_json::to_string(&err_handle(a)).unwrap();
                        resp_sender.send(WsMsg::Text(resp)).unwrap();
                    }
                    Err(_) => {
                        tracing::warn!(target: "Walle-core","json deserialize failed: {:?}", text)
                    }
                },
            },
            WsMsg::Binary(v) => match rmp_serde::from_read(v.as_slice()) {
                Ok(echo_action) => {
                    ok_handle(echo_action, true);
                }
                Err(_) => match rmp_serde::from_read(v.as_slice()) {
                    Ok(a) => {
                        let resp = rmp_serde::to_vec(&err_handle(a)).unwrap();
                        resp_sender.send(WsMsg::Binary(resp)).unwrap();
                    }
                    Err(_) => {
                        tracing::warn!(target: "Walle-core","rmp deserialize failed: {:?}", v)
                    }
                },
            },
            WsMsg::Ping(b) => {
                resp_sender.send(WsMsg::Pong(b)).unwrap();
            }
            WsMsg::Close(_) => {
                return true;
            }
            _ => {}
        }
        false
    }

    pub(crate) async fn ws(self: &Arc<Self>) -> WalleResult<()> {
        if !self.config.websocket.is_empty() {
            info!(target: "Walle-core", "Starting websocket reverse server.");
        }

        for wss in self.config.websocket.clone().into_iter() {
            let addr = std::net::SocketAddr::new(wss.host, wss.port);
            let tcp_listener = tokio::net::TcpListener::bind(&addr)
                .await
                .map_err(WalleError::from)?;
            info!(target: "Walle-core", "Websocket listening on {}", addr.to_string().red());
            let ob = self.clone();
            tokio::spawn(async move {
                ob.ws_hooks.on_start(&ob).await;
                while ob.is_running() {
                    if let Ok((stream, _)) = tcp_listener.accept().await {
                        if let Ok(ws_stream) =
                            crate::comms::ws_util::upgrade_websocket(&wss.access_token, stream)
                                .await
                        {
                            let ob = ob.clone();
                            tokio::spawn(async move {
                                // spawn to handle connect
                                ob.ws_loop(ws_stream).await.unwrap();
                            });
                        }
                    }
                }
                ob.ws_hooks.on_shutdown(&ob).await;
            });
            self.set_running();
        }
        Ok(())
    }

    pub(crate) async fn wsr(self: &Arc<Self>) {
        use crate::comms::util::AuthReqHeaderExt;

        for wsr in self.config.websocket_rev.clone().into_iter() {
            let ob = self.clone();
            tokio::spawn(async move {
                info!(target: "Walle-core", "Start try connect to {}", wsr.url.red());
                ob.ws_hooks.before_connect(&ob).await;
                while ob.is_running() {
                    let req = Request::builder()
                        .uri(&wsr.url)
                        .header(
                            USER_AGENT,
                            format!("OneBot/{} ({}) Walle/{}", V, ob.platform, crate::VERSION),
                        )
                        .header("X-OneBot-Version", V.to_string())
                        .header("X-Platform", ob.platform.clone())
                        .header("X-Impl", ob.r#impl.clone())
                        .header("X-Self-ID", ob.self_id.read().await.as_str())
                        .header("X-Client-Role", "Universal".to_string()) // for v11
                        .header_auth_token(&wsr.access_token)
                        .body(())
                        .unwrap();
                    match crate::comms::ws_util::try_connect(&wsr, req).await {
                        Ok(ws_stream) => ob.ws_loop(ws_stream).await.wran_err(),
                        Err(_) => {
                            warn!(target: "Walle-core", "Failed to connect to {}", wsr.url.red());
                            info!(target: "Walle-core", "Retry in {} seconds", wsr.reconnect_interval);
                            tokio::time::sleep(Duration::from_secs(wsr.reconnect_interval as u64))
                                .await;
                            ob.ws_hooks.before_reconnect(&ob).await;
                        }
                    }
                }
                ob.ws_hooks.on_shutdown(&ob).await;
            });
            self.set_running();
        }
    }
}
