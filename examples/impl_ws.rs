use std::sync::Arc;

use walle_core::config::ImplConfig;
use walle_core::impls::OneBot;
use walle_core::DefaultHandler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let ob = OneBot::new(
        "impl",
        "platform",
        "self_id",
        ImplConfig::default(),
        Arc::new(DefaultHandler),
    )
    .arc();
    ob.run().await.unwrap();
    loop {
        // block the main
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
