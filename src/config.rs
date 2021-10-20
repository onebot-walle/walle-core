use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    heartheat: bool,
    http: Vec<Http>,
    http_webhook: Vec<HttpWebhook>,
    websocket: Vec<WebSocket>,
    websocket_rev: Vec<WebSocketRev>,
}

#[derive(Debug, Deserialize)]
pub struct Http {
    host: std::net::IpAddr,
    port: u16,
    access_token: String,
    event_enable: bool,
    event_buffer_size: usize,
}

#[derive(Debug, Deserialize)]
pub struct HttpWebhook {
    url: String,
    access_token: String,
    timeout: u32,
}

#[derive(Debug, Deserialize)]
pub struct WebSocket {
    host: std::net::IpAddr,
    port: u16,
    access_token: String,
}

#[derive(Debug, Deserialize)]
pub struct WebSocketRev {
    url: String,
    access_token: String,
    reconnect_interval: u32,
}
