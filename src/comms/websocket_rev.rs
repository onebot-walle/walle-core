use serde::{de::DeserializeOwned, Serialize};
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::handshake::client::Request;

use crate::config::WebSocketRev;

#[cfg(feature = "impl")]
pub async fn run<E, A, R>(
    config: &WebSocketRev,
    broadcaster: crate::impls::CustomEventBroadcaster<E>,
    sender: crate::impls::CustomActionSender<A, R>,
) -> JoinHandle<()>
where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
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
        super::util::websocket_loop(ws_stream, broadcaster.subscribe(), sender.clone()).await;
    })
}

// #[cfg(feature = "sdk")]
// pub async fn sdk_run() {}
