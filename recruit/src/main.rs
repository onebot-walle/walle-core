pub(crate) mod core;
mod handle;

#[abras_onebot::tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("abras_onebot=trace");
    tracing_subscriber::fmt().with_env_filter(env).init();
    let mut bots = core::Bots::default();
    let bot = core::Bot::new(
        "recruit".to_owned(),
        abras_onebot::Config {
            heartheat: true,
            http: vec![],
            http_webhook: vec![],
            websocket: vec![abras_onebot::config::WebSocket {
                host: std::net::IpAddr::from([0, 0, 0, 0]),
                port: 8080,
                access_token: None,
            }],
            websocket_rev: vec![abras_onebot::config::WebSocketRev {
                url: "ws://192.168.99.99:8080".to_owned(),
                access_token: None,
                reconnect_interval: 100,
            }],
        },
    );
    bots.add_bot("recruit".to_owned(), bot).await;
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    bots.send_private_message(
        "recruit".to_owned(),
        "recruit",
        vec![abras_onebot::MessageSegment::Text {
            text: "hello world!".to_owned(),
        }],
    )
    .await;
}
