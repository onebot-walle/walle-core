#[cfg(any(feature = "http", feature = "websocket"))]
pub enum ContentTpye {
    Json,
    MsgPack,
}

#[cfg(any(feature = "http", feature = "websocket"))]
impl ContentTpye {
    pub fn new(s: &str) -> Option<Self> {
        match s {
            "application/json" => Some(Self::Json),
            "application/msgpack" => Some(Self::MsgPack),
            _ => None,
        }
    }
}

#[cfg(feature = "websocket")]
use std::sync::Arc;
#[cfg(feature = "websocket")]
use tokio::{net::TcpStream, sync::RwLock, task::JoinHandle};

#[cfg(feature = "websocket")]
pub struct WebSocketServer {
    pub listner: JoinHandle<()>,
    pub conns: Arc<RwLock<Vec<JoinHandle<()>>>>,
}

#[cfg(feature = "websocket")]
impl WebSocketServer {
    pub(crate) async fn abort(self) {
        let mut conns = self.conns.write().await;
        for conn in conns.iter_mut() {
            conn.abort();
        }
        self.listner.abort();
    }
}

#[cfg(feature = "websocket")]
pub(crate) async fn try_connect(
    config: &crate::config::WebSocketRev,
) -> Option<tokio_tungstenite::WebSocketStream<TcpStream>> {
    use tracing::{error, info};
    let ws_url = format!("ws://{}", config.url);
    let mut req =
        tokio_tungstenite::tungstenite::handshake::client::Request::builder().uri(&ws_url);
    if let Some(token) = &config.access_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let req = req.body(()).unwrap();
    match tokio::net::TcpStream::connect(&config.url).await {
        Ok(tcp_stream) => match tokio_tungstenite::client_async(req, tcp_stream).await {
            Ok((ws_stream, _)) => {
                info!(target: "Walle-core", "success connect to {}", ws_url);
                Some(ws_stream)
            }
            Err(e) => {
                error!(target: "Walle-core", "upgrade connect to ws error {}", e);
                None
            }
        },
        Err(e) => {
            error!(target: "Walle-core", "connect ws server error {}", e);
            None
        }
    }
}
