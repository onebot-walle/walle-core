#[cfg(any(feature = "http", feature = "websocket"))]
#[allow(dead_code)]
pub enum ContentTpye {
    Json,
    MsgPack,
}

#[cfg(any(feature = "http", feature = "websocket"))]
impl ContentTpye {
    #[allow(dead_code)]
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
use tokio_tungstenite::WebSocketStream;

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
    config: &crate::config::WebSocketClient,
) -> Option<tokio_tungstenite::WebSocketStream<TcpStream>> {
    use tokio_tungstenite::tungstenite::http::Uri;
    use tracing::{error, info};

    let uri: Uri = config.url.parse().unwrap();
    let addr = format!("{}:{}", uri.host().unwrap(), uri.port().unwrap());
    let ws_url = format!("ws://{}", addr);
    let mut req =
        tokio_tungstenite::tungstenite::handshake::client::Request::builder().uri(&ws_url);
    if let Some(token) = &config.access_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let req = req.body(()).unwrap();
    match tokio::net::TcpStream::connect(&addr).await {
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

#[cfg(feature = "websocket")]
pub(crate) async fn upgrade_websocket(
    access_token: &Option<String>,
    stream: TcpStream,
) -> Option<WebSocketStream<TcpStream>> {
    use tokio_tungstenite::tungstenite::{
        handshake::client::{Request, Response},
        http::Response as HttpResp,
    };
    use tracing::{error, info};

    let addr = match stream.peer_addr() {
        Ok(a) => a,
        Err(_) => {
            error!("connectting streams should have a peer address");
            return None;
        }
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
        Ok(s) => {
            info!(target:"Walle-core","new ws connect from {}",addr);
            Some(s)
        }
        Err(e) => {
            error!(target:"Walle-core","ws upgrade fail with error {}", e);
            None
        }
    }
}
