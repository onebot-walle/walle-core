use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    pub heartheat: bool,
    pub http: Vec<Http>,
    pub http_webhook: Vec<HttpWebhook>,
    pub websocket: Vec<WebSocket>,
    pub websocket_rev: Vec<WebSocketRev>,
}

#[derive(Debug, Deserialize)]
pub struct Http {
    pub host: std::net::IpAddr,
    pub port: u16,
    pub access_token: Option<String>,
    pub event_enable: bool,
    pub event_buffer_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct HttpWebhook {
    pub url: String,
    pub access_token: Option<String>,
    pub timeout: u64,
}

#[derive(Debug, Deserialize)]
pub struct WebSocket {
    pub host: std::net::IpAddr,
    pub port: u16,
    pub access_token: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketRev {
    pub url: String,
    pub access_token: Option<String>,
    pub reconnect_interval: u32,
}
