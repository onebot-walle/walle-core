use walle::{builtin::Echo, Plugins, Walle};
use walle_core::AppConfig;

#[tokio::main]
async fn main() {
    let plugins = Plugins::new().add_message_plugin(Echo::new());
    let walle = Walle::new(AppConfig::default(), plugins);
    walle.start().await.unwrap();
}
