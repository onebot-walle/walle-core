use walle_core::config::ImplConfig;
use walle_core::impls::StandardOneBot;
use walle_core::DefaultHandler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let ob = StandardOneBot::new(
        "impl",
        "platform",
        "self_id",
        ImplConfig::default(),
        DefaultHandler,
    )
    .arc();
    ob.run().await.unwrap();
    loop {
        // block the main
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
