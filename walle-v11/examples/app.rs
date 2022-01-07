use std::time::Duration;
use walle_core::{AppConfig, WebSocketServer};
use walle_v11::DefaultHandler;

#[tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("Walle-core=trace");
    tracing_subscriber::fmt().with_env_filter(env).init();
    let ob = walle_v11::app::OneBot11::new(
        AppConfig {
            websocket_rev: vec![WebSocketServer::default()],
            ..Default::default()
        },
        DefaultHandler::arc(),
    )
    .arc();
    ob.run().await.unwrap();
    tokio::time::sleep(Duration::from_secs(60)).await;
    ob.shutdown().await;
}
