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
    color_eyre::install()?;
    init_log();
        AppOBC::new(),
        TracingHandler::<Event, Action, Resp>::default(),
    ));
    ob.start(AppConfig::default(), (), true).await.unwrap();
    // ob.wait_all().await;
    ob.shutdown(true).await.ok();
fn init_log() {
    use tracing::Level;
    use tracing_subscriber::prelude::*;
    let filter = tracing_subscriber::filter::Targets::new()
        .with_target("Walle-core", Level::TRACE)
        .with_target("Walle-OBC", Level::TRACE)
        .with_default(Level::INFO);

    let registry = tracing_subscriber::registry()
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_timer(tracing_subscriber::fmt::time::LocalTime::new(
                    time::macros::format_description!(
                    "[year repr:last_two]-[month]-[day] UTC[offset_hour] [hour]:[minute]:[second]"
                ),
                )),
        )
        .with(filter);
    registry.init();
}
