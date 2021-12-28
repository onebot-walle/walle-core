use crate::{
    impls::CustomOneBot, Echo, EventContent, FromStandard, Resp, WalleError, WalleLogExt,
    WalleResult,
};
use futures_util::{SinkExt, StreamExt};
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc, time::Duration};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message as WsMsg, WebSocketStream};

type RespSender<R> = tokio::sync::mpsc::UnboundedSender<Echo<Resp<R>>>;

impl<E, A, R> CustomOneBot<E, A, R>
where
    E: FromStandard<EventContent> + Clone + Serialize + Send + 'static + Debug,
    A: DeserializeOwned + Send + 'static + Debug,
    R: Serialize + Send + 'static + Debug,
{
    pub(crate) async fn ws_loop(
        self: &Arc<Self>,
        mut ws_stream: WebSocketStream<TcpStream>,
    ) -> WalleResult<()> {
        self.ws_hooks.on_connect(&self).await;
        let mut listener = self.broadcaster.subscribe();
        let (resp_tx, mut resp_rx) = tokio::sync::mpsc::unbounded_channel();
        loop {
            tokio::select! {
                event = listener.recv() => {
                    match event {
                        Ok(event) => {
                            let event = serde_json::to_string(&event).unwrap();
                            if let Err(_) = ws_stream.send(WsMsg::Text(event)).await {
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
                        match ws_msg {
                            Ok(ws_msg) => self.ws_recv(ws_msg, &resp_tx).await?,
                            Err(_) => break,
                        }
                    }
                },
                resp = resp_rx.recv() => {
                    if let Some(resp) = resp {
                        let resp = serde_json::to_string(&resp).unwrap();
                        if let Err(_) = ws_stream.send(WsMsg::Text(resp)).await {
                            break;
                        }
                    }
                }
            }
        }
        self.ws_hooks.on_disconnect(&self).await;
        Ok(())
    }

    pub(crate) async fn ws_recv(
        self: &Arc<Self>,
        ws_msg: WsMsg,
        resp_sender: &RespSender<R>,
    ) -> WalleResult<()> {
        if let WsMsg::Text(text) = ws_msg {
            let msg: Echo<A> =
                serde_json::from_str(&text).map_err(|e| WalleError::SerdeJsonError(e))?;
            let (action, echo_s) = msg.unpack();
            let sender = resp_sender.clone();
            let ob = self.clone();
            tokio::spawn(async move {
                let resp = ob.action_handler.handle(action).await;
                let echo = echo_s.pack(resp);
                sender.send(echo).unwrap();
            });
        }
        Ok(())
    }

    pub(crate) async fn ws(self: &Arc<Self>) -> WalleResult<()> {
        for wss in self.config.websocket.clone().into_iter() {
            let addr = std::net::SocketAddr::new(wss.host, wss.port);
            let tcp_listener = tokio::net::TcpListener::bind(&addr)
                .await
                .map_err(|e| WalleError::TcpServerBindAddressError(e))?;
            let ob = self.clone();
            tokio::spawn(async move {
                ob.ws_hooks.on_start(&ob).await;
                while ob.is_running() {
                    if let Ok((stream, _)) = tcp_listener.accept().await {
                        if let Ok(ws_stream) =
                            crate::comms::ws_utils::upgrade_websocket(&wss.access_token, stream)
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
        for wsr in self.config.websocket_rev.clone().into_iter() {
            let ob = self.clone();
            tokio::spawn(async move {
                ob.ws_hooks.before_connect(&ob).await;
                while ob.is_running() {
                    match crate::comms::ws_utils::try_connect(&wsr).await {
                        Ok(ws_stream) => ob.ws_loop(ws_stream).await.log_err(),
                        Err(_) => {
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
