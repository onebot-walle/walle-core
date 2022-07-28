use std::sync::Arc;
use walle_core::action::Action;
use walle_core::alt::TracingHandler;
use walle_core::config::ImplConfig;
use walle_core::event::Event;
use walle_core::obc::ImplOBC;
use walle_core::resp::Resp;
use walle_core::OneBot;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let ob = Arc::new(OneBot::new_12(
        TracingHandler::<Event, Action, Resp>::default(),
        ImplOBC::new(
            "impl".to_string(),
            "platform".to_string(),
        ),
    ));
    let tasks = ob.start((), ImplConfig::default(), true).await.unwrap();
    for task in tasks {
        task.await.unwrap()
    }
}
