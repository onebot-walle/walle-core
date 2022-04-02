use walle::{
    builtin::{Echo, Echo2},
    Walle,
};
use walle_core::AppConfig;

#[tokio::main]
async fn main() {
    let walle = Walle::new(AppConfig::default(), vec![Echo::new(), Echo2::new()].into());
    walle.start().await.unwrap();
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}
