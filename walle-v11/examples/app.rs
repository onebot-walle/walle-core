use walle_core::{AppConfig, WebSocketClient};
use walle_v11::DefaultHandler;

#[tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("Walle-core=trace");
    tracing_subscriber::fmt().with_env_filter(env).init();
    let ob = walle_v11::app::OneBot11::new(
        AppConfig {
            websocket: vec![WebSocketClient::default()],
            websocket_rev: vec![],
            ..Default::default()
        },
        DefaultHandler::arc(),
    )
    .arc();
    ob.run_block().await.unwrap();
}
