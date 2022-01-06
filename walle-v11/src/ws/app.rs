use crate::{app::OneBot, utils::EventOrResp};
use futures_util::StreamExt;
use std::{
    collections::HashMap,
    sync::{atomic::Ordering, Arc},
    time::Duration,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message as WsMsg, WebSocketStream};
use walle_core::EchoS;

impl OneBot {
    pub(crate) async fn websocket_loop(self: Arc<Self>, mut ws_stream: WebSocketStream<TcpStream>) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
        let mut bot_ids: Vec<i32> = vec![]; // 记录本连接上的 bot_id
        let mut echo_map = HashMap::new();
        while self.running.load(Ordering::SeqCst) {
            tokio::select! {
                action = rx.recv() => {
                    if let Some((action, tx)) = action {
                        let echo_s = EchoS::new("v11action");
                        let echo_action = echo_s.pack(action);
                        super::ws_action_send(&mut ws_stream, echo_action).await;
                        echo_map.insert(echo_s, tx);
                    }
                },
                msg = ws_stream.next() => {
                    if let Some(msg) = msg {
                        match msg {
                            Ok(msg) => self.ws_recv(msg, &tx, &mut bot_ids, &mut echo_map).await,
                            Err(_) => {
                                break;
                            }
                        }
                    }
                }
            }
        }
        // 断开连接后，移除本连接的 bot
        for bot_id in bot_ids {
            self.remove_bot(bot_id).await;
        }
    }

    pub(crate) async fn ws_recv(
        self: &Arc<Self>,
        msg: WsMsg,
        tx: &crate::app::ActionSender,
        bot_ids: &mut Vec<i32>,
        echo_map: &mut HashMap<EchoS, crate::app::RespSender>,
    ) {
        if let WsMsg::Text(text) = msg {
            let income_item: EventOrResp = serde_json::from_str(&text).unwrap();
            match income_item {
                EventOrResp::Event(event) => match self.get_bot(event.self_id).await {
                    Some(bot) => self.handler.handle(bot, event).await,
                    None => {
                        let bot = self.add_bot(event.self_id, tx.clone()).await;
                        bot_ids.push(event.self_id);
                        self.handler.handle(bot, event).await;
                    }
                },
                EventOrResp::Resp(resp) => {
                    let (resp, echo_s) = resp.unpack();
                    if let Some(tx) = echo_map.remove(&echo_s) {
                        tx.send(resp).unwrap();
                    }
                }
            }
        }
    }

    pub(crate) async fn ws(self: &Arc<Self>) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        for wsc in self.config.websocket.clone().into_iter() {
            let ob = self.clone();
            self.running.store(true, Ordering::SeqCst);
            tokio::spawn(async move {
                if ob.running.load(Ordering::SeqCst) {
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

    pub(crate) async fn wsr(self: &Arc<Self>) {
        if self.running.load(Ordering::SeqCst) {
            return;
        }
        for wsr in self.config.websocket_rev.clone().into_iter() {
            let addr = std::net::SocketAddr::new(wsr.host, wsr.port);
            let tcp_listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
            let ob = self.clone();
            self.running.store(true, Ordering::SeqCst);
            tokio::spawn(async move {
                while let Ok((stream, _)) = tcp_listener.accept().await {
                    if let Some(ws_stream) =
                        super::upgrade_websocket(&wsr.access_token, stream).await
                    {
                        tokio::spawn(ob.clone().websocket_loop(ws_stream));
                    }
                }
            });
        }
    }
}
