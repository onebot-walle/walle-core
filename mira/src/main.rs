use walle_core::{app::OneBot, config::AppConfig, DefaultHandler};

#[tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("walle_core=trace,Walle-core=info");
    tracing_subscriber::fmt().with_env_filter(env).init();
    let config = AppConfig::default();
    let cli = OneBot::new(config, DefaultHandler::arc(), DefaultHandler::arc());
    cli.run().await.unwrap();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await
    }
}
