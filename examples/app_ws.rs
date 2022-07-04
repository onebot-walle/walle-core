use std::sync::Arc;

use walle_core::alt::TracingHandler;
use walle_core::config::AppConfig;
use walle_core::obc::AppOBC;
use walle_core::prelude::*;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let ob = Arc::new(OneBot::new_12(
        AppOBC::new(),
        TracingHandler::<StandardEvent, StandardAction, StandardResps>::default(),
    ));
    let joins = ob.start(AppConfig::default(), (), true).await.unwrap();
    for join in joins {
        join.await.unwrap()
    }
}
