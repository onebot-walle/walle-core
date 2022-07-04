use std::sync::Arc;

use walle_core::config::ImplConfig;
use walle_core::obc::ImplOBC;
use walle_core::prelude::*;
use walle_core::util::TracingHandler;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let ob = Arc::new(OneBot::new_12(
        TracingHandler::<StandardEvent, StandardEvent, StandardResps>::default(),
        ImplOBC::new(
            "self_id".to_string(),
            "impl".to_string(),
            "platform".to_string(),
        ),
    ));
    let joins = ob.start((), ImplConfig::default(), true).await.unwrap();
    for join in joins {
        join.await.unwrap()
    }
}
