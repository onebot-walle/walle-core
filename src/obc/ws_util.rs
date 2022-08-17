use super::OBC;
use colored::*;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::handshake::client::{generate_key, Request, Response};
use tokio_tungstenite::tungstenite::http::{
    request::Builder as HttpReqBuilder, response::Builder as HttpRespBuilder, Response as HttpResp,
    Uri,
};
use tokio_tungstenite::{accept_hdr_async, client_async, WebSocketStream};
use tracing::{info, warn};

use crate::config::WebSocketClient;

pub(crate) async fn try_connect(
    config: &WebSocketClient,
    req: HttpReqBuilder,
) -> Option<WebSocketStream<TcpStream>> {
    fn err<E: std::fmt::Display>(
        config: &WebSocketClient,
        e: E,
    ) -> Option<WebSocketStream<TcpStream>> {
        warn!(target: OBC, "connect to {} failed: {}", config.url, e);
        info!(
            target: OBC,
            "Retry in {} seconds", config.reconnect_interval
        );
        None
    }
    let uri: Uri = config.url.parse().unwrap();
    let addr = format!("{}:{}", uri.host().unwrap(), uri.port().unwrap());
    let authority = match uri.authority() {
        Some(authority) => authority.as_str(),
        None => return err(config, "authority is empty"),
    };
    let host = authority
        .find('@')
        .map(|idx| authority.split_at(idx + 1).1)
        .unwrap_or_else(|| authority);

    let stream = match TcpStream::connect(&addr).await {
        Ok(stream) => stream,
        Err(e) => return err(config, e),
    };

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
        stream,
    )
    .await
    {
        Ok((ws_stream, _)) => {
            info!(target: OBC, "Success connect to {}", config.url);
            Some(ws_stream)
        }
        Err(e) => err(config, e),
    }
}

pub(crate) async fn upgrade_websocket(
    access_token: &Option<String>,
    stream: TcpStream,
) -> Option<(WebSocketStream<TcpStream>, String)> {
    let addr = match stream.peer_addr() {
        Ok(addr) => addr,
        Err(e) => {
            warn!(target: OBC, "Upgrade websocket failed: {}", e);
            return None;
        }
    };
    let mut implt = String::default();
    let ref_implt = &mut implt;

    let callback = |req: &Request, resp: Response| -> Result<Response, HttpResp<Option<String>>> {
        use crate::obc::check_query;
        let headers = req.headers();
        if let Some(token) = access_token {
            if let Some(auth) = headers.get("Authorization").and_then(|a| a.to_str().ok()) {
                if auth.strip_prefix("Bearer ") != Some(token) {
                    return Err(HttpRespBuilder::new()
                        .status(403)
                        .body(Some("Authorization Header is invalid".to_string()))
                        .unwrap());
                }
            } else if let Some(auth) = check_query(req.uri()) {
                if auth != token {
                    return Err(HttpRespBuilder::new()
                        .status(403)
                        .body(Some("Authorization Query is invalid".to_string()))
                        .unwrap());
                }
            } else {
                return Err(HttpRespBuilder::new()
                    .status(403)
                    .body(Some("Missing Authorization Header".to_string()))
                    .unwrap());
            }
        }
        if let Some(Some((version, implt))) = headers
            .get("Sec-WebSocket-Protocol")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.split_once('.'))
        {
            if version == "12" {
                *ref_implt = implt.to_owned();
            }
        }
        info!(
            target: OBC,
            "Websocket connectted with {}",
            addr.to_string().blue()
        );
        Ok(resp)
    };

    match accept_hdr_async(stream, callback).await {
        Ok(s) => {
            info!(target: OBC, "New websocket client connected from {}", addr);
            Some((s, implt))
        }
        Err(e) => {
            info!(target: OBC, "Upgrade websocket from {} failed: {}", addr, e);
            None
        }
    }
}
