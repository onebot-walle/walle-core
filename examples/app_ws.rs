use std::sync::Arc;
use walle_core::action::Action;
use walle_core::alt::TracingHandler;
use walle_core::config::AppConfig;
use walle_core::event::Event;
use walle_core::obc::AppOBC;
use walle_core::resp::Resp;
use walle_core::OneBot;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let ob = Arc::new(OneBot::new_12(
        AppOBC::new(),
        TracingHandler::<Event, Action, Resp>::default(),
    ));
    let tasks = ob.start(AppConfig::default(), (), true).await.unwrap();
    for task in tasks {
        task.await.unwrap()
    }
}
