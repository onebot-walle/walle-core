use walle::{
    plugins::{Echo, Echo2},
    Matcher, Walle,
};
use walle_core::AppConfig;

#[tokio::main]
async fn main() {
    let walle = Walle::new(AppConfig::default(), vec![Echo::new(), Echo2::new()].into());
}
