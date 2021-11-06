use clap::Parser;

#[derive(Parser)]
#[clap(name = "mira", version = "0.1.0", author = "Abrahum")]
/// A OneBot v12 test client
pub(crate) struct Root {
    /// log debug level
    #[clap(short, long)]
    pub(crate) debug: bool,
    /// log trace level
    #[clap(short, long)]
    pub(crate) trace: bool,
    /// set access_token
    #[clap(short, long)]
    pub(crate) access_token: Option<String>,
    /// set ws url (default: ws://127.0.0.1:8844)
    #[clap(long)]
    pub(crate) ws: Option<String>,
    /// set reconnect_interval (default: 4)
    #[clap(short, long)]
    pub(crate) reconnect_interval: Option<u32>,
    /// wsr address(if set ws, wsr will be ignored)
    #[clap(long)]
    pub(crate) wsr: Option<std::net::SocketAddr>,
    /// config file path (file > mira.toml > default)
    #[clap(short, long)]
    pub(crate) config: Option<String>,
    /// output default config to mira.toml
    #[clap(long)]
    pub(crate) output_config: bool,
}
