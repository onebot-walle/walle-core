//! OBC 配置项

use std::{collections::HashMap, fmt::Debug};

use serde::{Deserialize, Serialize};

/// OneBot 实现端设置项
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ImplConfig {
    pub http: Vec<HttpServer>,
    pub http_webhook: Vec<HttpClient>,
    pub websocket: Vec<WebSocketServer>,
    pub websocket_rev: Vec<WebSocketClient>,
    pub heartbeat: Heartbeat,
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
#[derive(Debug, Deserialize, Serialize, Clone)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub http_webhook: Vec<HttpServer>,
    pub websocket: Vec<WebSocketClient>,
    pub websocket_rev: Vec<WebSocketServer>,
    pub http: HashMap<String, HttpClient>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            http: HashMap::default(),
            http_webhook: vec![],
            websocket: vec![],
            websocket_rev: vec![WebSocketServer::default()],
        }
    }
}

impl AppConfig {
    pub fn empty() -> Self {
        Self {
            http: HashMap::default(),
            http_webhook: vec![],
            websocket: vec![],
            websocket_rev: vec![],
        }
    }
}

/// OneBot Impl Http 通讯设置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpServer {
    pub host: std::net::IpAddr,
    pub port: u16,
    pub access_token: Option<String>,
    #[cfg(feature = "impl")]
    pub event_enable: bool,
    #[cfg(feature = "impl")]
    pub event_buffer_size: usize,
}

impl Default for HttpServer {
    fn default() -> Self {
        Self {
            host: std::net::IpAddr::from([127, 0, 0, 1]),
            port: 6700,
            access_token: None,
            #[cfg(feature = "impl")]
            event_enable: true,
            #[cfg(feature = "impl")]
            event_buffer_size: 16,
        }
    }
}

/// OneBot Impl Http Webhook 通讯设置
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpClient {
    #[serde(rename = "impl")]
    pub implt: Option<String>,
    pub platform: Option<String>,
    pub url: String,
    pub access_token: Option<String>,
    pub timeout: u64,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self {
            implt: None,
            platform: None,
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

#[test]
fn toml_test() {
    let config = AppConfig::default();
    println!("{:?}", toml::to_string(&config));
}
