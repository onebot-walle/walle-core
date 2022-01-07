use futures_util::{SinkExt, StreamExt};
use hyper::header::USER_AGENT;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::HashMap, fmt::Debug, sync::Arc};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{mpsc, oneshot, RwLock},
};
use tokio_tungstenite::{
    tungstenite::{error::Result as WsResult, Message as WsMsg},
    WebSocketStream,
};

use crate::{
    app::{CustomActionSender, CustomOneBot, CustomRespSender},
    BasicEvent, Echo, EchoS, WalleError, WalleLogExt, WalleResult,
};

impl<E, A, R, const V: u8> CustomOneBot<E, A, R, V>
where
    E: BasicEvent + Clone + DeserializeOwned + Send + 'static + Debug,
    A: Clone + Serialize + Send + 'static + Debug,
    R: Clone + DeserializeOwned + Send + 'static + Debug,
{
    async fn ws_loop(
        self: &Arc<Self>,
        mut ws_stream: WebSocketStream<TcpStream>,
    ) -> WalleResult<()> {
        self.ws_hooks.on_connect(&self).await;
        let (action_tx, mut action_rx) = mpsc::unbounded_channel();
        let mut bot_ids: Vec<String> = vec![];
        let echo_map = RwLock::default();
        while self.is_running() {
            tokio::select! {
                action = action_rx.recv() => {
                    if let Some((action,tx)) = action {
                        if let Err(_) = self.ws_send_action(&mut ws_stream, action, tx, &echo_map).await {
                            break;
                        }
                    }
                },
                msg = ws_stream.next() => {
                    if let Some(msg) = msg {
                        match msg {
                            Ok(msg) => {
                                self.ws_recv(msg, &mut bot_ids,&action_tx, &echo_map).await.wran_err();
                            }
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }
        }
        for bot_id in bot_ids {
            self.remove_bot(&bot_id).await;
        }
        self.ws_hooks.on_disconnect(&self).await;
        Ok(())
    }

    async fn ws_send_action(
        &self,
        ws_stream: &mut WebSocketStream<TcpStream>,
        action: A,
        sender: CustomRespSender<R>,
        echo_map: &RwLock<HashMap<EchoS, CustomRespSender<R>>>,
    ) -> WsResult<()> {
        let echo_s = EchoS::new("action");
        echo_map.write().await.insert(echo_s.clone(), sender);
        let action = echo_s.pack(action);
        let action = serde_json::to_string(&action).unwrap();
        ws_stream.send(WsMsg::Text(action)).await
    }

    async fn ws_recv(
        self: &Arc<Self>,
        ws_msg: WsMsg,
        bot_ids: &mut Vec<String>,
        action_tx: &CustomActionSender<A, R>,
        echo_map: &RwLock<HashMap<EchoS, oneshot::Sender<R>>>,
    ) -> WalleResult<()> {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        enum ReceiveItem<E, R> {
            Event(E),
            Resp(Echo<R>),
        }

        if let WsMsg::Text(text) = ws_msg {
            let item: ReceiveItem<E, R> =
                serde_json::from_str(&text).map_err(|e| WalleError::SerdeJsonError(e))?;
            match item {
                ReceiveItem::Event(event) => {
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
                ReceiveItem::Resp(resp) => {
                    let (resp, echos) = resp.unpack();
                    if let Some(rx) = echo_map.write().await.remove(&echos) {
                        rx.send(resp).unwrap();
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) async fn ws(self: &Arc<Self>) {
        use crate::comms::util::AuthReqHeaderExt;
        use tokio_tungstenite::tungstenite::http::Request;

        for wsc in self.config.websocket.clone().into_iter() {
            // info!(target: "Walle-core", "Running WebSocket");
            let ob = self.clone();
            tokio::spawn(async move {
                ob.ws_hooks.before_connect(&ob).await;
                while ob.is_running() {
                    let req = Request::builder()
                        .uri(&wsc.url)
                        .header(USER_AGENT, format!("OneBot/{} Walle-App/0.1.0", V))
                        .header_auth_token(&wsc.access_token)
                        .body(())
                        .unwrap();
                    match crate::comms::ws_util::try_connect(&wsc, req).await {
                        Ok(ws_stream) => {
                            ob.clone().ws_loop(ws_stream).await.wran_err();
                        }
                        Err(_) => {
                            tokio::time::sleep(std::time::Duration::from_secs(
                                wsc.reconnect_interval as u64,
                            ))
                            .await;
                            ob.ws_hooks.before_reconnect(&ob).await;
                        }
                    }
                }
                ob.ws_hooks.on_shutdown(&ob).await;
            });
        }
    }

    pub(crate) async fn wsr(self: &Arc<Self>) -> WalleResult<()> {
        for wss in self.config.websocket_rev.clone().into_iter() {
            // info!(target: "Walle-core", "Running WebSocket Reverse");
            let addr = std::net::SocketAddr::new(wss.host, wss.port);
            let tcp_listener = TcpListener::bind(&addr)
                .await
                .map_err(|e| WalleError::TcpServerBindAddressError(e))?;
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
                            tokio::spawn(async move { ob.ws_loop(ws_stream).await });
                        }
                    }
                }
                ob.ws_hooks.on_shutdown(&ob).await;
            });
        }
        Ok(())
    }
}
