pub(crate) mod core;
mod handle;

use walle_core::{config::ImplConfig, MessageSegment};

#[tokio::main]
async fn main() {
    let env = tracing_subscriber::EnvFilter::from("walle_core=trace,Walle-core=info");
    tracing_subscriber::fmt().with_env_filter(env).init();
    let mut bots = core::Bots::default();
    let bot = core::Bot::new("recruit".to_owned(), ImplConfig::default());
    bots.add_bot("recruit".to_owned(), bot).await;
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        bots.send_private_message(
            "recruit".to_owned(),
            vec![MessageSegment::Text {
                text: "hello world!".to_owned(),
            }],
            "hello world!".to_owned(),
        )
        .await;
    }
}
