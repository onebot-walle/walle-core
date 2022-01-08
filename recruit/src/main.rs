pub(crate) mod core;
mod handle;

use clap::Parser;
use walle_core::config::ImplConfig;

#[tokio::main]
async fn main() {
    let root = Root::parse();
    let env = tracing_subscriber::EnvFilter::from(if root.trace {
        "trace"
    } else if root.debug {
        "debug"
    } else {
        "info"
    });
    tracing_subscriber::fmt().with_env_filter(env).init();
    let mut bots = core::Bots::default();
    let bot = core::Bot::new("recruit", ImplConfig::default());
    bots.add_bot("recruit".to_owned(), bot).await;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        // bots.send_private_message(
        //     "recruit".to_owned(),
        //     vec![MessageSegment::Text {
        //         text: "hello world!".to_owned(),
        //     }],
        //     "hello world!".to_owned(),
        // )
        // .await;
    }
}

#[derive(Parser)]
#[clap(name = "recruit", version = "0.1.0", author = "Abrahum")]
pub(crate) struct Root {
    /// log debug level
    #[clap(short, long)]
    pub(crate) debug: bool,
    /// log trace level
    #[clap(short, long)]
    pub(crate) trace: bool,
}
