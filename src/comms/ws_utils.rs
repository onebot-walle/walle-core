use std::sync::Arc;
use tokio::{net::TcpStream, sync::RwLock, task::JoinHandle};
use tokio_tungstenite::{
    accept_hdr_async, client_async,
    tungstenite::{
        handshake::client::{Request, Response},
        http::{Response as HttpResp, Uri},
    },
    WebSocketStream,
};
use tracing::{error, info};

use crate::{WalleError, WalleResult};

pub struct WebSocketServer {
    pub listner: JoinHandle<()>,
    pub conns: Arc<RwLock<Vec<JoinHandle<()>>>>,
}

impl WebSocketServer {
    pub(crate) async fn abort(self) {
        let mut conns = self.conns.write().await;
        for conn in conns.iter_mut() {
            conn.abort();
        }
        self.listner.abort();
    }
}

pub(crate) async fn try_connect(
    config: &crate::config::WebSocketClient,
) -> WalleResult<WebSocketStream<TcpStream>> {
    let uri: Uri = config.url.parse().unwrap();
    let addr = format!("{}:{}", uri.host().unwrap(), uri.port().unwrap());
    let ws_url = format!("ws://{}", addr);
    let mut req = Request::builder().uri(&ws_url);
    if let Some(token) = &config.access_token {
        req = req.header("Authorization", format!("Bearer {}", token));
    }
    let req = req.body(()).unwrap();
    match client_async(
        req,
        TcpStream::connect(&addr).await.map_err(|e| {
            error!(target: "Walle-core", "connect ws server error {}", e);
            WalleError::TcpConnectFailed
        })?,
    )
    .await
    {
        Ok((ws_stream, _)) => {
            info!(target: "Walle-core", "success connect to {}", ws_url);
            Ok(ws_stream)
        }
        Err(e) => {
            error!(target: "Walle-core", "upgrade connect to ws error {}", e);
            Err(WalleError::WebsocketUpgradeFail)
        }
    }
}

pub(crate) async fn upgrade_websocket(
    access_token: &Option<String>,
    stream: TcpStream,
) -> WalleResult<WebSocketStream<TcpStream>> {
    let addr = stream.peer_addr().map_err(|_| {
        error!("connectting streams should have a peer address");
        WalleError::WebsocketNoAddress
    })?;

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

    match accept_hdr_async(stream, callback).await {
        Ok(s) => {
            info!(target:"Walle-core","new ws connect from {}",addr);
            Ok(s)
        }
        Err(e) => {
            error!(target:"Walle-core","ws upgrade fail with error {}", e);
            Err(WalleError::WebsocketUpgradeFail)
        }
    }
}
