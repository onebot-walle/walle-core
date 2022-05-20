use colored::*;
use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_hdr_async, client_async,
    tungstenite::{
        handshake::client::{generate_key, Request, Response},
        http::{
            request::Builder as HttpReqBuilder, response::Builder as HttpRespBuilder,
            Response as HttpResp, Uri,
        },
    },
    WebSocketStream,
};
use tracing::info;

use crate::{WalleRtError, WalleLogExt, WalleRtResult};

pub(crate) async fn try_connect(
    config: &crate::config::WebSocketClient,
    req: HttpReqBuilder,
) -> WalleRtResult<WebSocketStream<TcpStream>> {
    let uri: Uri = config.url.parse().unwrap();
    let addr = format!("{}:{}", uri.host().unwrap(), uri.port().unwrap());
    let authority = uri
        .authority()
        .ok_or_else(|| WalleRtError::UrlError(uri.to_string()))?
        .as_str();
    let host = authority
        .find('@')
        .map(|idx| authority.split_at(idx + 1).1)
        .unwrap_or_else(|| authority);

    match client_async(
        req.method("GET")
            .header("Host", host)
            .header("Connection", "Upgrade")
            .header("Upgrade", "websocket")
            .header("Sec-WebSocket-Version", "13")
            .header("Sec-WebSocket-Key", generate_key())
            .uri(uri)
            .body(())
            .unwrap(),
        TcpStream::connect(&addr)
            .await
            .map_err(WalleRtError::TcpConnectFailed)?,
    )
    .await
    {
        Ok((ws_stream, _)) => Ok(ws_stream).info(&format!("success connect to {}", config.url)),
        Err(e) => Err(WalleRtError::WebsocketUpgradeFail(e)),
    }
}

pub(crate) async fn upgrade_websocket(
    access_token: &Option<String>,
    stream: TcpStream,
) -> WalleRtResult<WebSocketStream<TcpStream>> {
    let addr = stream
        .peer_addr()
        .map_err(|_| WalleRtError::WebsocketNoAddress)?;

    let callback = |req: &Request, resp: Response| -> Result<Response, HttpResp<Option<String>>> {
        let headers = req.headers();
        if let Some(token) = access_token {
            match headers.get("Authorization").and_then(|a| a.to_str().ok()) {
                Some(auth) => {
                    if auth.strip_prefix("Bearer ") != Some(token) {
                        return Err(HttpRespBuilder::new()
                            .status(403)
                            .body(Some("Authorization Header is invalid".to_string()))
                            .unwrap());
                    }
                }
                None => {
                    return Err(HttpRespBuilder::new()
                        .status(403)
                        .body(Some("Missing Authorization Header".to_string()))
                        .unwrap())
                }
            }
        }
        info!(target: "Walle-core", "Websocket connectted with {}", addr.to_string().blue());
        Ok(resp)
    };

    match accept_hdr_async(stream, callback).await {
        Ok(s) => Ok(s).info(&format!("new websocket connectted from {}", addr)),
        Err(e) => Err(WalleRtError::WebsocketUpgradeFail(e)),
    }
}
