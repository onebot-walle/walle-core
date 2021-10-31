use walle_core::{config::SdkConfig, sdk::OneBot, DefaultHandler};

#[tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("walle_core=trace");
    tracing_subscriber::fmt().with_env_filter(env).init();
    let config = SdkConfig::default();
    let cli = OneBot::new(config, DefaultHandler::arc(), DefaultHandler::arc());
    cli.run().await;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await
    }
}
