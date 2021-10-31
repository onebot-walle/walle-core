use serde::Deserialize;

/// OneBot 实现端设置项
#[derive(Debug, Deserialize)]
pub struct ImplConfig {
    pub heartheat: bool,
    pub http: Vec<Http>,
    pub http_webhook: Vec<HttpWebhook>,
    pub websocket: Vec<WebSocket>,
    pub websocket_rev: Vec<WebSocketRev>,
}

/// OneBot 应用端设置项
#[derive(Debug, Deserialize)]
pub struct SdkConfig {
    pub http: Option<Http>,
    pub http_webhook: Option<HttpWebhook>,
    pub websocket: Option<WebSocketRev>,
    pub websocket_rev: Option<WebSocket>,
}

/// OneBot Impl Http 通讯设置
#[derive(Debug, Deserialize)]
pub struct Http {
    pub host: std::net::IpAddr,
    pub port: u16,
    pub access_token: Option<String>,
    pub event_enable: bool,
    pub event_buffer_size: usize,
}

/// OneBot Impl Http Webhook 通讯设置
#[derive(Debug, Deserialize)]
pub struct HttpWebhook {
    pub url: String,
    pub access_token: Option<String>,
    pub timeout: u64,
}

/// OneBot Impl 正向 WebSocket 通讯设置
#[derive(Debug, Deserialize)]
pub struct WebSocket {
    pub host: std::net::IpAddr,
    pub port: u16,
    pub access_token: Option<String>,
}

/// OneBot Impl 反向 WebSocket 通讯设置
#[derive(Debug, Clone, Deserialize)]
pub struct WebSocketRev {
    pub url: String,
    pub access_token: Option<String>,
    pub reconnect_interval: u32,
}

impl Default for ImplConfig {
    fn default() -> Self {
        Self {
            heartheat: true,
            http: vec![],
            http_webhook: vec![],
            websocket: vec![WebSocket {
                host: std::net::IpAddr::from([127, 0, 0, 1]),
                port: 8844,
                access_token: None,
            }],
            websocket_rev: vec![],
        }
    }
}

impl Default for SdkConfig {
    fn default() -> Self {
        Self {
            http: None,
            http_webhook: None,
            websocket: Some(WebSocketRev {
                url: "127.0.0.1:8844".to_owned(),
                access_token: None,
                reconnect_interval: 3,
            }),
            websocket_rev: None,
        }
    }
}
