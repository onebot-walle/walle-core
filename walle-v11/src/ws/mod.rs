use futures_util::SinkExt;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    client_async,
    tungstenite::{
        handshake::client::{Request, Response},
        http::{Response as HttpResp, Uri},
        Message as WsMsg,
    },
    WebSocketStream,
};
use walle_core::Echo;

pub(crate) mod app;
pub(crate) mod impls;

pub(crate) async fn try_connect(
    wsc: &walle_core::config::WebSocketClient,
) -> Option<WebSocketStream<TcpStream>> {
    let uri: Uri = wsc.url.parse().unwrap();
    let addr = format!("{}:{}", uri.host().unwrap(), uri.port().unwrap());
    let ws_uri = format!("ws://{}", addr);
    let mut req = Request::builder().uri(&ws_uri);
    if let Some(token) = &wsc.access_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let req = req.body(()).unwrap();
    match TcpStream::connect(&addr).await {
        Ok(tcp_stream) => match client_async(req, tcp_stream).await {
            Ok((ws_stream, _)) => Some(ws_stream),
            Err(_) => None,
        },
        Err(_) => None,
    }
}

pub(crate) async fn upgrade_websocket(
    access_token: &Option<String>,
    stream: TcpStream,
) -> Option<WebSocketStream<TcpStream>> {
    let _addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(_) => return None,
    };

    let callback = |req: &Request, resp: Response| -> Result<Response, HttpResp<Option<String>>> {
        let headers = req.headers();
        match access_token {
            Some(token) => match headers.get("Authorization") {
                Some(get_token) => {
                    if get_token == token {
                        Ok(resp)
                    } else {
                        Err(HttpResp::new(None))
                    }
                }
                None => Err(HttpResp::new(None)),
            },
            None => Ok(resp),
        }
    };

    match tokio_tungstenite::accept_hdr_async(stream, callback).await {
        Ok(s) => Some(s),
        Err(_) => None,
    }
}

pub(crate) async fn ws_send(ws_stream: &mut WebSocketStream<TcpStream>, msg: WsMsg) {
    ws_stream.send(msg).await.unwrap();
}

pub(crate) async fn ws_event_send(
    ws_stream: &mut WebSocketStream<TcpStream>,
    event: crate::event::Event,
) {
    ws_send(
        ws_stream,
        WsMsg::Text(serde_json::to_string(&event).unwrap()),
    )
    .await
}

pub(crate) async fn ws_action_send(
    ws_stream: &mut WebSocketStream<TcpStream>,
    action: walle_core::Echo<crate::action::Action>,
) {
    ws_send(
        ws_stream,
        WsMsg::Text(serde_json::to_string(&action).unwrap()),
    )
    .await
}

pub(crate) async fn ws_resp_send(
    ws_stream: &mut WebSocketStream<TcpStream>,
    resp: Echo<crate::action::Resp>,
) {
    ws_send(
        ws_stream,
        WsMsg::Text(serde_json::to_string(&resp).unwrap()),
    )
    .await
}

// pub(crate) async fn ws_recv(
//     msg: WsMsg,
//     action_handler: &ArcActionHandler,
//     event_handler: &ArcEventHandler,
// ) {
//     #[derive(Debug, Deserialize, Serialize)]
//     enum IncomeItem {
//         Event(crate::event::Event),
//         Action(crate::action::Action),
//         Resp(crate::action::Resp),
//     }
//     if let WsMsg::Text(msg_str) = msg {
//         let income_item: IncomeItem = serde_json::from_str(&msg_str).unwrap();
//         match income_item {
//             IncomeItem::Event(event) => {
//                 event_handler.handle(event).await;
//             }
//             IncomeItem::Action(action) => {
//                 action_handler.handle(action).await;
//             }
//             IncomeItem::Resp(resp) => {
//                 event_handler.handle_resp(resp).await;
//             }
//         }
//     }
// }

// pub(crate) async fn ws(
//     config: walle_core::config::WebSocket,
//     tool: BoxTool,
// ) -> UnboundedReceiver<UnboundedSender<Action>> {
//     let addr = std::net::SocketAddr::new(config.host, config.port);
//     let tcp_listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
//     let (tx, rx) = tokio::sync::mpsc::unbounded_channel();
//     tokio::spawn(async move {
//         while let Ok((stream, _)) = tcp_listener.accept().await {
//             if let Some(ws_stream) = upgrade_websocket(&config.access_token, stream).await {
//                 let (action_tx, action_rx) = tokio::sync::mpsc::unbounded_channel();
//                 tokio::spawn(websocketloop(ws_stream, tool.clone(), action_rx));
//                 tx.send(action_tx).unwrap();
//             }
//         }
//     });
//     rx
// }

// pub(crate) async fn wsr(
//     config: walle_core::config::WebSocketRev,
//     tool: BoxTool,
// ) -> UnboundedReceiver<UnboundedSender<Action>> {
//     todo!()
// }
