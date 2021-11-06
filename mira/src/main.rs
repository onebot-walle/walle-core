use std::path::PathBuf;

use clap::Parser;
use tracing::{info, warn};
use walle_core::{
    app::OneBot,
    config::{AppConfig, WebSocket, WebSocketRev},
    DefaultHandler, Message, MessageBuild,
};

mod root;
mod shell;

static CONFIG_FILE: &str = "mira.toml";

#[tokio::main]
async fn main() {
    let root = root::Root::parse();
    if root.output_config {
        new_config_file();
        return;
    }
    let env = tracing_subscriber::EnvFilter::from(if root.trace {
        "trace"
    } else if root.debug {
        "debug"
    } else {
        "info"
    });
    tracing_subscriber::fmt().with_env_filter(env).init();
    let config = if let Some(url) = root.ws {
        let mut config = AppConfig::empty();
        let ws = WebSocketRev {
            url,
            access_token: root.access_token,
            reconnect_interval: if let Some(interval) = root.reconnect_interval {
                interval
            } else {
                4
            },
        };
        config.websocket = Some(ws);
        config
    } else if let Some(addr) = root.wsr {
        let mut config = AppConfig::empty();
        let wsr = WebSocket {
            host: addr.ip(),
            port: addr.port(),
            access_token: root.access_token,
        };
        config.websocket_rev = Some(wsr);
        config
    } else {
        load_config(root.config)
    };
    let cli = OneBot::new(config, DefaultHandler::arc()).arc();
    OneBot::run(cli.clone()).await.unwrap();
    loop {
        let stdin = std::io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        input = input.replace("\r", "");
        input.remove(input.len() - 1);
        if !input.is_empty() {
            cli.send_message(
                "private".to_owned(),
                None,
                Some("recruit".to_owned()),
                Message::new().text(input),
            )
            .await;
        }
    }
}

fn new_config_file() {
    let path = PathBuf::from(CONFIG_FILE);
    if !path.exists() {
        let config = AppConfig::default();
        info!("saving default config to {}", CONFIG_FILE);
        std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
    } else {
        info!(
            "file {} exist, please delete or rename it first",
            CONFIG_FILE
        );
    }
}

fn load_config(path: Option<String>) -> AppConfig {
    let path = if let Some(p) = &path {
        p.as_str()
    } else {
        CONFIG_FILE
    };
    let path = PathBuf::from(path);

    if !path.exists() {
        warn!(
            "{} dont't exists using default config",
            path.to_str().unwrap()
        );
        let config = AppConfig::default();
        return config;
    }

    if let Ok(s) = std::fs::read_to_string(&path) {
        match toml::from_str::<AppConfig>(&s) {
            Ok(c) => {
                info!("using config form {}", path.to_str().unwrap());
                c
            }
            Err(e) => {
                warn!(
                    "loading config file {} error: {}",
                    path.to_str().unwrap(),
                    e
                );
                warn!("using default config");
                AppConfig::default()
            }
        }
    } else {
        warn!(
            "open file {} fail, using default config",
            path.to_str().unwrap()
        );
        AppConfig::default()
    }
}
