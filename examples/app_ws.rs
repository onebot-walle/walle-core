use color_eyre::eyre::Result;
use std::sync::Arc;
use walle_core::action::Action;
use walle_core::alt::{ColoredAlt, TracingHandler};
use walle_core::config::{AppConfig, WebSocketServer};
use walle_core::event::Event;
use walle_core::obc::AppOBC;
use walle_core::resp::Resp;
use walle_core::WalleResult;
use walle_core::{ActionHandler, EventHandler, OneBot};
use tokio::task::JoinHandle;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    init_log();
    let ob = Arc::new(OneBot::new(AppOBC::new(), MyEventHandler));
    let mut config = AppConfig::default();
    config.websocket_rev.clear();
    let ws_server = WebSocketServer {
        host: std::net::IpAddr::from([0, 0, 0, 0]),
        port: 2456,
        access_token: None,
    };
    config.websocket_rev.push(ws_server);
    ob.start(config, (), true).await.unwrap();
    tokio::spawn(async {
        let ob = ob;
        ob.handle_action(action)
    });
    ob.wait_all().await;
    ob.shutdown(true).await.ok();
    Ok(())
}
#[derive(Debug)]
struct MyEventHandler;
impl EventHandler<Event, Action, Resp> for MyEventHandler {
    type Config = ();
    async fn start<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH>>,
        _config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        Ok(vec![])
    }
    async fn call<AH, EH>(
        &self,
        event: Event,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> walle_core::WalleResult<()>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        tracing::info!("{}", event.colored_alt());
        Ok(())
    }
    async fn shutdown(&self) {
        tracing::info!("Shutting down TracingHandler")
    }

    fn on_onebot_connect<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> impl futures_util::Future<Output = WalleResult<()>>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        async { Ok(()) }
    }

    fn on_onebot_disconnect<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> impl futures_util::Future<Output = WalleResult<()>>
    where
        AH: ActionHandler<Event, Action, Resp> + Send + Sync + 'static,
        EH: EventHandler<Event, Action, Resp> + Send + Sync + 'static,
    {
        async { Ok(()) }
    }
}

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
