mod bots;
mod handler;
mod prelude;

const CONFIG_PATH: &str = "WalleHub.toml";

#[tokio::main]
async fn main() {
    let config = bots::UnionConfig::load_or_new(CONFIG_PATH);
    let bot = bots::UnionBot::new(config);
    bot.run().await;
}
