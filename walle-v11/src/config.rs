use serde::{Deserialize, Serialize};
use walle_core::config::{WebSocket, WebSocketRev};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub web_socket: Option<WebSocketRev>,
    pub web_socket_rev: Option<WebSocket>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImplConfig {
    pub web_socket: Option<WebSocket>,
    pub web_socket_rev: Option<WebSocketRev>,
}
