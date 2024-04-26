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
    let ob = Arc::new(OneBot::new(
        TracingHandler::<Event, Action, Resp>::default(),
        ImplOBC::new("impl".to_string()),
        walle_core::structs::Version {
            implt: walle_core::WALLE_CORE.to_owned(),
            version: walle_core::VERSION.to_owned(),
            onebot_version: 12.to_string(),
        },
    ));
    ob.start((), ImplConfig::default(), true).await.unwrap();
    // ob.wait_all().await;
    ob.shutdown(true).await.ok();
}
