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
use tokio::net::TcpStream;

#[cfg(feature = "websocket")]
pub(crate) async fn try_connect(
    config: &crate::config::WebSocketRev,
) -> Option<tokio_tungstenite::WebSocketStream<TcpStream>> {
    use tracing::error;
    let mut req =
        tokio_tungstenite::tungstenite::handshake::client::Request::builder().uri(&config.url);
    if let Some(token) = &config.access_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let req = req.body(()).unwrap();
    match tokio::net::TcpStream::connect(&config.url).await {
        Ok(tcp_stream) => match tokio_tungstenite::client_async(req, tcp_stream).await {
            Ok((ws_stream, _)) => Some(ws_stream),
            Err(e) => {
                error!("upgrade connect to ws error {}", e);
                None
            }
        },
        Err(e) => {
            error!("connect ws server error {}", e);
            None
        }
    }
}

// #[cfg(all(feature = "websocket", feature = "impl"))]
// pub(crate) async fn sdk_websocket_loop(ws_stream: tokio_tungstenite::WebSocketStream<TcpStream>) {

// }
