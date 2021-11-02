use clap::Parser;

#[derive(Parser)]
#[clap(name = "mira", version = "0.1.0", author = "Abrahum")]
/// A OneBot v12 test client
pub(crate) struct Root {
    #[clap(subcommand)]
    pub(crate) sub_comand: Commands,
}

#[derive(Parser)]
pub(crate) enum Commands {
    /// Run clinet with Websocket
    Ws(Ws),
    /// Run client with Websocket-Rev
    Wsr(Wsr),
    /// Run from config file
    Run(Run),
}

#[derive(Parser)]
pub(crate) struct Ws {
    /// url for websocket
    pub url: Option<String>,
    #[clap(short, long)]
    /// access_token
    pub access_token: Option<String>,
    #[clap(short, long)]
    /// reconnect_interval
    pub reconnect_interval: Option<u32>,
}

#[derive(Parser)]
pub(crate) struct Wsr {
    #[clap(short, long)]
    /// host ip
    pub ip: Option<std::net::IpAddr>,
    /// port
    #[clap(short, long)]
    pub port: Option<u16>,
    /// access_token
    #[clap(short, long)]
    pub access_token: Option<String>,
}

#[derive(Parser)]
pub(crate) struct Run {
    #[clap(short, long)]
    /// Config file path
    pub(crate) config: Option<String>,
}
