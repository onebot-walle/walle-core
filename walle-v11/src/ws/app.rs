use crate::{action::Action, app::OneBot, utils::EventOrResp};
use futures_util::StreamExt;
use std::{
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use tokio::{net::TcpStream, task::JoinHandle};
use tokio_tungstenite::{tungstenite::Message as WsMsg, WebSocketStream};

impl OneBot {
    pub(crate) async fn websocket_loop(self: Arc<Self>, mut ws_stream: WebSocketStream<TcpStream>) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut self_id: Option<i32> = None;
        loop {
            tokio::select! {
                action = rx.recv() => {
                    if let Some(action) = action {
                        super::ws_action_send(&mut ws_stream, action).await;
                    }
                },
                msg = ws_stream.next() => {
                    if let Some(msg) = msg {
                        match msg {
                            Ok(msg) => self.ws_recv(msg, &tx, &mut self_id).await,
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }
        }
        if let Some(id) = self_id {
            self.remove_bot(id).await;
        }
    }

    pub(crate) async fn ws_recv(
        self: &Arc<Self>,
        msg: WsMsg,
        tx: &tokio::sync::mpsc::UnboundedSender<Action>,
        self_id: &mut Option<i32>,
    ) {
        if let WsMsg::Text(text) = msg {
            let income_item: EventOrResp = serde_json::from_str(&text).unwrap();
            match income_item {
                EventOrResp::Event(event) => match self.get_bot(event.self_id).await {
                    Some(bot) => self.handler.handle(bot, event).await,
                    None => {
                        let bot = self.add_bot(event.self_id, tx.clone()).await;
                        self_id.replace(event.self_id);
                        self.handler.handle(bot, event).await;
                    }
                },
                EventOrResp::Resp(resp) => self.handler.handle_resp(self.clone(), resp).await,
            }
        }
    }

    pub(crate) async fn ws(self: &Arc<Self>) -> Option<JoinHandle<()>> {
        if self.running.load(Ordering::SeqCst) {
            return None;
        }
        if let Some(wsc) = self.config.web_socket.clone() {
            let ob = self.clone();
            self.running.store(true, Ordering::SeqCst);
            Some(tokio::spawn(async move {
                loop {
                    match super::try_connect(&wsc).await {
                        Some(ws_stream) => ob.clone().websocket_loop(ws_stream).await,
                        None => {
                            tokio::time::sleep(Duration::from_secs(wsc.reconnect_interval as u64))
                                .await
                        }
                    }
                }
            }))
        } else {
            None
        }
    }

    pub(crate) async fn wsr(self: &Arc<Self>) -> Option<JoinHandle<()>> {
        if self.running.load(Ordering::SeqCst) {
            return None;
        }
        if let Some(wsr) = (self.config.web_socket_rev).clone() {
            let addr = std::net::SocketAddr::new(wsr.host, wsr.port);
            let tcp_listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            let ob = self.clone();
            self.running.store(true, Ordering::SeqCst);
            Some(tokio::spawn(async move {
                while let Ok((stream, _)) = tcp_listener.accept().await {
                    if let Some(ws_stream) =
                        super::upgrade_websocket(&wsr.access_token, stream).await
                    {
                        tokio::spawn(ob.clone().websocket_loop(ws_stream));
                    }
                }
            }))
        } else {
            None
        }
    }
}
