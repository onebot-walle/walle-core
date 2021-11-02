use std::path::PathBuf;

use clap::Parser;
use walle_core::{app::OneBot, config::AppConfig, DefaultHandler};

mod root;

static CONFIG_FILE: &str = "mira.toml";

#[tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("walle_core=trace,Walle-core=info");
    tracing_subscriber::fmt().with_env_filter(env).init();
    let root = root::Root::parse();
    let config = load_or_new(root.config);
    let cli = OneBot::new(config, DefaultHandler::arc());
    cli.run().await.unwrap();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await
    }
}

fn load_or_new(path: Option<String>) -> AppConfig {
    let path = if let Some(p) = &path { p } else { CONFIG_FILE };
    let path = PathBuf::from(path);

    if !path.exists() {
        let config = AppConfig::default();
        std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
        return config;
    }

    if let Ok(s) = std::fs::read_to_string(&path) {
        if let Ok(c) = toml::from_str::<AppConfig>(&s) {
            return c;
        }
    }

    AppConfig::default()
}
