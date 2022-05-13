use walle_core::app::StandardOneBot;
use walle_core::config::AppConfig;
use walle_core::DefaultHandler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let ob = StandardOneBot::new(AppConfig::default(), DefaultHandler).arc();
    ob.run_block().await.unwrap();
}
