use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, sync::Arc, vec};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot},
    task::JoinHandle,
};
use tokio_tungstenite::{
    tungstenite::{error::Result as WsResult, http::header::USER_AGENT, Message as WsMsg},
    WebSocketStream,
};
use tracing::{info, warn};

use crate::{
    app::{CustomActionSender, CustomRespSender, OneBot},
    Echo, EchoS, ProtocolItem, SelfId, WalleError, WalleLogExt, WalleResult,
};

impl<E, A, R, const V: u8> OneBot<E, A, R, V>
where
    E: ProtocolItem + SelfId + Clone + Send + 'static + Debug,
    A: ProtocolItem + Clone + Send + 'static + Debug,
    R: ProtocolItem + Clone + Send + 'static + Debug,
{
    async fn ws_loop(
        self: &Arc<Self>,
        mut ws_stream: WebSocketStream<TcpStream>,
    ) -> WalleResult<()> {
        self.ws_hooks.on_connect(self).await;
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        let mut bot_ids: Vec<String> = vec![];
        let mut echo_map = HashMap::default();
        while self.is_running() {
            tokio::select! {
                action = action_rx.recv() => {
                    if let Some((action,tx)) = action {
                        if self.ws_send_action(&mut ws_stream, action, tx, &mut echo_map).await.is_err() {
                            break;
                        }
                    }
                },
                msg = ws_stream.next() => {
                    if let Some(msg) = msg {
                        match msg {
                            Ok(msg) => {
                                if self.ws_recv(msg, &mut bot_ids, &action_tx, &mut echo_map, &mut ws_stream).await {
                                    break;
                                }
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }
        }
        ws_stream.send(WsMsg::Close(None)).await.ok();
        for bot_id in bot_ids {
            self.remove_bot(&bot_id).await;
        }
        self.ws_hooks.on_disconnect(self).await;
        Ok(())
    }

    async fn ws_send_action(
        &self,
        ws_stream: &mut WebSocketStream<TcpStream>,
        action: A,
        sender: CustomRespSender<R>,
        echo_map: &mut HashMap<EchoS, CustomRespSender<R>>,
    ) -> WsResult<()> {
        let echo_s = EchoS::new("action");
        echo_map.insert(echo_s.clone(), sender);
        let action = echo_s.pack(action);
        let action = action.json_encode();
        ws_stream.send(WsMsg::Text(action)).await
    }

    async fn ws_recv(
        self: &Arc<Self>,
        ws_msg: WsMsg,
        bot_ids: &mut Vec<String>,
        action_tx: &CustomActionSender<A, R>,
        echo_map: &mut HashMap<EchoS, oneshot::Sender<R>>,
        ws_stream: &mut WebSocketStream<TcpStream>,
    ) -> bool {
        #[derive(Debug, Deserialize, Serialize)]
        #[serde(untagged)]
        enum ReceiveItem<E, R> {
            Event(E),
            Resp(Echo<R>),
        }

        let handle_ok = |item: Result<ReceiveItem<E, R>, String>| async move {
            match item {
                Ok(ReceiveItem::Event(event)) => {
                    let ob = self.clone();
                    let self_id = event.self_id();
                    let bot = match self.get_bot(&self_id).await {
                        Some(bot) => bot,
                        None => {
                            bot_ids.push(self_id.to_string());
                            ob.insert_bot(&self_id, action_tx).await
                        }
                    };
                    tokio::spawn(async move { ob.event_handler.handle(bot, event).await });
                }
                Ok(ReceiveItem::Resp(resp)) => {
                    let (resp, echos) = resp.unpack();
                    if let Some(rx) = echo_map.remove(&echos) {
                        rx.send(resp).unwrap();
                    }
                }
                Err(s) => warn!(target: "Walle-core", "serde failed: {}", s),
            }
        };

        match ws_msg {
            WsMsg::Text(text) => handle_ok(ProtocolItem::json_decode(&text)).await,
            WsMsg::Binary(bin) => handle_ok(ProtocolItem::rmp_decode(&bin)).await,
            WsMsg::Ping(b) => {
                if let Err(_) = ws_stream.send(WsMsg::Pong(b)).await {
                    return true;
                }
            }
            WsMsg::Pong(_) => {}
            WsMsg::Close(_) => return true,
            WsMsg::Frame(_) => unreachable!(),
        }
        false
    }

    pub(crate) async fn ws(self: &Arc<Self>) -> Vec<JoinHandle<()>> {
        use crate::comms::utils::AuthReqHeaderExt;
        use tokio_tungstenite::tungstenite::http::Request;

        let mut joins = vec![];
        for wsc in self.config.websocket.clone().into_iter() {
            info!(target: "Walle-core", "Start try connect to {}", wsc.url);
            let ob = self.clone();
            joins.push(tokio::spawn(async move {
                ob.ws_hooks.before_connect(&ob).await;
                while ob.is_running() {
                    let req = Request::builder()
                        .uri(&wsc.url)
                        .header(
                            USER_AGENT,
                            format!("OneBot/{} Walle-App/{}", V, crate::VERSION),
                        )
                        .header_auth_token(&wsc.access_token)
                        .body(())
                        .unwrap();
                    match crate::comms::ws_utils::try_connect(&wsc, req).await {
                        Ok(ws_stream) => {
                            info!(target: "Walle-core", "Successfully connected to {}", wsc.url);
                            ob.clone().ws_loop(ws_stream).await.wran_err();
                            warn!(target: "Walle-core", "Disconnected from {}", wsc.url);
                        }
                        Err(_) => {
                            warn!(target: "Walle-core", "Failed to connect to {}", wsc.url);
                            warn!(target: "Walle-core", "Retry in {} seconds", wsc.reconnect_interval);
                            tokio::time::sleep(std::time::Duration::from_secs(
                                wsc.reconnect_interval as u64,
                            ))
                            .await;
                            ob.ws_hooks.before_reconnect(&ob).await;
                        }
                    }
                }
                ob.ws_hooks.on_shutdown(&ob).await;
            }))
        }
        joins
    }

    pub(crate) async fn wsr(self: &Arc<Self>) -> WalleResult<Vec<JoinHandle<()>>> {
        let mut joins = vec![];
        for wss in self.config.websocket_rev.clone().into_iter() {
            // info!(target: "Walle-core", "Running WebSocket Reverse");
            let addr = std::net::SocketAddr::new(wss.host, wss.port);
            info!(target: "Walle-core", "Starting Websocket server in {}", addr);
            let tcp_listener = TcpListener::bind(&addr)
                .await
                .map_err(WalleError::TcpServerBindAddressError)?;
            let ob = self.clone();
            joins.push(tokio::spawn(async move {
                info!(target: "Walle-core", "Websocket server started");
                ob.ws_hooks.on_start(&ob).await;
                while ob.is_running() {
                    if let Ok((stream, _)) = tcp_listener.accept().await {
                        if let Ok(ws_stream) =
                            crate::comms::ws_utils::upgrade_websocket(&wss.access_token, stream)
                                .await
                        {
                            let ob = ob.clone();
                            tokio::spawn(async move { ob.ws_loop(ws_stream).await });
                        }
                    }
                }
                info!(target: "Walle-core", "Websocket server shutting down");
                ob.ws_hooks.on_shutdown(&ob).await;
            }))
        }
        Ok(joins)
    }
}
