use crate::{
    action::{Action, Resp},
    impls::OneBot,
};
use futures_util::StreamExt;
use walle_core::Echo;
use std::sync::{atomic::Ordering, Arc};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message as WsMsg, WebSocketStream};

impl OneBot {
    /// 接收 WsMsg 解析为 Action 并 spawn 处理
    pub(crate) async fn ws_recv(
        self: &Arc<Self>,
        msg: WsMsg,
        sender: tokio::sync::mpsc::UnboundedSender<Echo<Resp>>,
    ) {
        if let WsMsg::Text(text) = msg {
            let action: Echo<Action> = serde_json::from_str(&text).unwrap();
            let ob = self.clone();
            tokio::spawn(async move {
                let (action, echo_s) = action.unpack();
                let resp = ob.handler.handle(action).await;
                let resp = echo_s.pack(resp);
                sender.send(resp).unwrap();
            });
        }
    }

    /// 启动一个 WsLoop
    pub(crate) async fn websocket_loop(self: Arc<Self>, mut ws_stream: WebSocketStream<TcpStream>) {
        let (resp_tx, mut resp_rx) = tokio::sync::mpsc::unbounded_channel();
        let mut event_rx = self.sender.subscribe();
        while self.running.load(Ordering::SeqCst) {
            tokio::select! {
                event = event_rx.recv() => {
                    if let Ok(event) = event {
                        super::ws_event_send(&mut ws_stream, event).await;
                    }
                },
                resp = resp_rx.recv() => {
                    if let Some(resp) = resp {
                        super::ws_resp_send(&mut ws_stream, resp).await;
                    }
                },
                action = ws_stream.next() => {
                    if let Some(action) = action {
                        if let Ok(action) = action {
                            self.ws_recv(action, resp_tx.clone()).await;
                        } else {
                            break;
                        }
                    }
                }
            }
        }
    }

    /// 启动正向 Ws 服务器
    pub(crate) async fn ws(self: &Arc<Self>) {
        for wss in self.config.websocket.clone().into_iter() {
            let addr = std::net::SocketAddr::new(wss.host, wss.port);
            let tcp_listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            let ob = self.clone();
            self.running.store(true, Ordering::SeqCst);
            tokio::spawn(async move {
                while let Ok((stream, _)) = tcp_listener.accept().await {
                    if let Some(ws_stream) =
                        super::upgrade_websocket(&wss.access_token, stream).await
                    {
                        tokio::spawn(ob.clone().websocket_loop(ws_stream));
                    }
                }
            });
        }
    }

    /// 启动反向 Ws 客户端
    pub(crate) async fn wsr(self: &Arc<Self>) {
        for wsc in self.config.websocket_rev.clone().into_iter() {
            let ob = self.clone();
            self.running.store(true, Ordering::SeqCst);
            tokio::spawn(async move {
                while ob.running.load(Ordering::SeqCst) {
                    match super::try_connect(&wsc).await {
                        Some(ws_stream) => ob.clone().websocket_loop(ws_stream).await,
                        None => {
                            tokio::time::sleep(Duration::from_secs(wsc.reconnect_interval as u64))
                                .await
                        }
                    }
                }
            });
        }
    }
}
