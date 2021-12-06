use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_hdr_async, client_async,
    tungstenite::{
        handshake::client::{Request, Response},
        http::{Response as HttpResp, Uri},
    },
    WebSocketStream,
};

use crate::{WalleError, WalleLogExt, WalleResult};

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
        TcpStream::connect(&addr)
            .await
            .map_err(|e| WalleError::TcpConnectFailed(e))?,
    )
    .await
    {
        Ok((ws_stream, _)) => Ok(ws_stream).info(format!("success connect to {}", ws_url)),
        Err(e) => Err(WalleError::WebsocketUpgradeFail(e)),
    }
}

pub(crate) async fn upgrade_websocket(
    access_token: &Option<String>,
    stream: TcpStream,
) -> WalleResult<WebSocketStream<TcpStream>> {
    let addr = stream
        .peer_addr()
        .map_err(|_| WalleError::WebsocketNoAddress)?;

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
        Ok(s) => Ok(s).info(format!("new websocket connectted from {}", addr)),
        Err(e) => Err(WalleError::WebsocketUpgradeFail(e)),
    }
}
