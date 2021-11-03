use std::path::PathBuf;

use clap::Parser;
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
    let env = tracing_subscriber::EnvFilter::from("walle_core=trace,Walle-core=debug,mira=info");
    tracing_subscriber::fmt().with_env_filter(env).init();
    let root = root::Root::parse();
    let config = match root.sub_comand {
        root::Commands::Run(r) => load_or_new(r.config),
        root::Commands::Ws(ws) => {
            let mut wsc = WebSocketRev::default();
            then_do(&ws.url, &mut wsc.url);
            then_do(&ws.reconnect_interval, &mut wsc.reconnect_interval);
            wsc.access_token = ws.access_token;
            let mut config = AppConfig::empty();
            config.websocket = Some(wsc);
            config
        }
        root::Commands::Wsr(wsr) => {
            let mut wsrc = WebSocket::default();
            then_do(&wsr.ip, &mut wsrc.host);
            then_do(&wsr.port, &mut wsrc.port);
            wsrc.access_token = wsr.access_token;
            let mut config = AppConfig::empty();
            config.websocket_rev = Some(wsrc);
            config
        }
    };
    let cli = OneBot::new(config, DefaultHandler::arc()).arc();
    OneBot::run(cli.clone()).await.unwrap();
    loop {
        let stdin = std::io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();
        // println!("in:{}", input);
        cli.send_message(
            "private".to_owned(),
            None,
            Some("recruit".to_owned()),
            Message::new().text(input),
        )
        .await;
        // println!("{}", input);
        // tokio::time::sleep(std::time::Duration::from_secs(1)).await
    }
}

fn then_do<T>(t: &Option<T>, o: &mut T)
where
    T: Clone,
{
    if let Some(t) = t {
        *o = t.clone();
    }
}

fn load_or_new(path: Option<String>) -> AppConfig {
    use tracing::{info, warn};

    let (path, default) = if let Some(p) = &path {
        (p.as_str(), false)
    } else {
        (CONFIG_FILE, true)
    };
    let path = PathBuf::from(path);

    if !path.exists() {
        warn!(
            "{} dont't exists using default config",
            path.to_str().unwrap()
        );
        let config = AppConfig::default();
        if default {
            info!("saving default config to {}", CONFIG_FILE);
            std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
        }
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
