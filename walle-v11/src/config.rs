use serde::{Deserialize, Serialize};
use walle_core::config::{WebSocketClient, WebSocketServer};

#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub web_socket: Option<WebSocketClient>,
    pub web_socket_rev: Option<WebSocketServer>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ImplConfig {
    pub web_socket: Option<WebSocketServer>,
    pub web_socket_rev: Option<WebSocketClient>,
}
