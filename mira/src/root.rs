use clap::Parser;

#[derive(Parser)]
#[clap(name = "mira", version = "0.1.0", author = "Abrahum")]
/// A OneBot v12 test client
pub(crate) struct Root {
    #[clap(short, long)]
    /// Config file path
    pub(crate) config: Option<String>,
    #[clap(subcommand)]
    pub(crate) sub_comand: Commands,
}

#[derive(Parser)]
pub(crate) enum Commands {
    /// Run clinet with Websocket
    Ws(Ws),
    /// Run client with Websocket-Rev
    Wsr(Wsr),
}

#[derive(Parser)]
pub(crate) struct Ws {}

#[derive(Parser)]
pub(crate) struct Wsr {}
