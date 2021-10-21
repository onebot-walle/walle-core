use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use crate::config::WebSocketRev;

pub async fn run(
    config: &WebSocketRev,
    broadcaster: crate::EventBroadcaster,
    sender: crate::ActionSender,
) -> JoinHandle<()> {
    let url = config.url.clone();
    let access_token = config.access_token.clone();
    let _reconnect_interval = config.reconnect_interval;
    tokio::spawn(async move {
        let req = Request::builder().uri(url.clone());
        let req = if let Some(token) = access_token {
            req.header("Authorization", format!("Bearer {}", token))
        } else {
            req
        };
        let req = req.body(()).unwrap();
        let tcp_stream = tokio::net::TcpStream::connect(url).await.unwrap();
        let (ws_stream, _) = tokio_tungstenite::client_async(req, tcp_stream)
            .await
            .unwrap();
        super::util::web_socket_loop(ws_stream, broadcaster.subscribe(), sender.clone()).await;
    })
}
