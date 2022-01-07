use std::fmt::Debug;

use serde::{Deserialize, Serialize};

/// OneBot 实现端设置项
#[derive(Debug, Deserialize, Serialize)]
pub struct ImplConfig {
    pub heartbeat: Heartbeat,
    pub http: Vec<Http>,
    pub http_webhook: Vec<HttpWebhook>,
    pub websocket: Vec<WebSocketServer>,
    pub websocket_rev: Vec<WebSocketClient>,
}

impl Default for ImplConfig {
    fn default() -> Self {
        Self {
            heartbeat: Heartbeat::default(),
            http: vec![],
            http_webhook: vec![],
            websocket: vec![],
            websocket_rev: vec![WebSocketClient::default()],
        }
    }
}

/// OneBot 心跳设置
///
/// 间隔为 0 则默认为 4
#[derive(Debug, Deserialize, Serialize)]
pub struct Heartbeat {
    pub enabled: bool,
    pub interval: u32,
}

impl Default for Heartbeat {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: 4,
        }
    }
}

/// OneBot 应用端设置项
#[derive(Debug, Deserialize, Serialize)]
pub struct AppConfig {
    pub http: Vec<Http>,
    pub http_webhook: Vec<HttpWebhook>,
    pub websocket: Vec<WebSocketClient>,
    pub websocket_rev: Vec<WebSocketServer>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            http: vec![],
            http_webhook: vec![],
            websocket: vec![],
            websocket_rev: vec![WebSocketServer::default()],
        }
    }
}

impl AppConfig {
    pub fn empty() -> Self {
        Self {
            http: vec![],
            http_webhook: vec![],
            websocket: vec![],
            websocket_rev: vec![],
        }
    }
}

/// OneBot Impl Http 通讯设置
#[derive(Debug, Deserialize, Serialize)]
pub struct Http {
    pub host: std::net::IpAddr,
    pub port: u16,
    pub access_token: Option<String>,
    pub event_enable: bool,
    pub event_buffer_size: usize,
}

impl Default for Http {
    fn default() -> Self {
        Self {
            host: std::net::IpAddr::from([127, 0, 0, 1]),
            port: 6700,
            access_token: None,
            event_enable: true,
            event_buffer_size: 16,
        }
    }
}

/// OneBot Impl Http Webhook 通讯设置
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpWebhook {
    pub url: String,
    pub access_token: Option<String>,
    pub timeout: u64,
}

impl Default for HttpWebhook {
    fn default() -> Self {
        Self {
            url: "http://127.0.0.1:6700".to_owned(),
            access_token: None,
            timeout: 4,
        }
    }
}

/// OneBot WebSocket 服务器设置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebSocketServer {
    pub host: std::net::IpAddr,
    pub port: u16,
    pub access_token: Option<String>,
}

impl Default for WebSocketServer {
    fn default() -> Self {
        Self {
            host: std::net::IpAddr::from([127, 0, 0, 1]),
            port: 8844,
            access_token: None,
        }
    }
}

/// OneBot Impl 反向 WebSocket 通讯设置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebSocketClient {
    pub url: String,
    pub access_token: Option<String>,
    pub reconnect_interval: u32,
}

impl Default for WebSocketClient {
    fn default() -> Self {
        Self {
            url: "ws://127.0.0.1:8844".to_owned(),
            access_token: None,
            reconnect_interval: 4,
        }
    }
}
