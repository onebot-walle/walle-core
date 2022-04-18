use std::sync::Arc;

use walle_core::{AppConfig, WalleResult};

pub mod builtin;
mod matcher;
mod plugin;
pub use matcher::*;
pub use plugin::*;

pub struct Walle {
    pub ob: Arc<walle_core::app::StandardOneBot>,
}

impl Walle {
    pub fn new(config: AppConfig, plugins: Plugins) -> Self {
        let timer = tracing_subscriber::fmt::time::OffsetTime::new(
            time::UtcOffset::from_hms(8, 0, 0).unwrap(),
            time::format_description::parse(
                "[year repr:last_two]-[month]-[day] [hour]:[minute]:[second]",
            )
            .unwrap(),
        );
        let env = tracing_subscriber::EnvFilter::from("debug");
        tracing_subscriber::fmt()
            .with_env_filter(env)
            .with_timer(timer)
            .init();
        Self {
            ob: Arc::new(walle_core::app::StandardOneBot::new(
                config,
                Box::new(plugins),
            )),
        }
    }

    pub async fn start(self) -> WalleResult<()> {
        self.ob.run().await
    }
}
