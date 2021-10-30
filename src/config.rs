use serde::Deserialize;

/// Config for OneBot Impl
#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub heartheat: bool,
    pub http: Vec<Http>,
    pub http_webhook: Vec<HttpWebhook>,
    pub websocket: Vec<WebSocket>,
    pub websocket_rev: Vec<WebSocketRev>,
}

#[derive(Debug, Deserialize, Default)]
pub struct SdkConfig {
    pub heartheat: bool,
    pub http: Vec<Http>,
    pub http_webhook: Vec<HttpWebhook>,
    pub websocket: Vec<WebSocketRev>,
    pub websocket_rev: Vec<WebSocket>,
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
