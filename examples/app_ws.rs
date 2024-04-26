use std::sync::Arc;
use walle_core::alt::TracingHandler;
use walle_core::config::AppConfig;
use walle_core::obc::AppOBC;
use walle_core::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let ob = Arc::new(OneBot::new(
        AppOBC::new(),
        TracingHandler::<Event, Action, Resp>::default(),
        Version {
            implt: walle_core::WALLE_CORE.to_owned(),
            version: walle_core::VERSION.to_owned(),
            onebot_version: 12.to_string(),
        },
    ));
    ob.start(AppConfig::default(), (), true).await.unwrap();
    // ob.wait_all().await;
    ob.shutdown(true).await.ok();
}
