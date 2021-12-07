use std::sync::{atomic::AtomicBool, Arc};

use crate::ArcActionHandler;

pub struct OneBot {
    pub(crate) r#impl: String,
    pub(crate) platform: String,
    pub(crate) handler: ArcActionHandler,
    pub(crate) sender: tokio::sync::broadcast::Sender<crate::event::Event>,
    pub(crate) running: AtomicBool,
    pub(crate) config: walle_core::ImplConfig,
}

impl OneBot {
    pub fn new(
        r#impl: String,
        platform: String,
        handler: ArcActionHandler,
        sender: tokio::sync::broadcast::Sender<crate::event::Event>,
        config: walle_core::ImplConfig,
    ) -> Self {
        Self {
            r#impl,
            platform,
            handler,
            sender,
            running: AtomicBool::new(false),
            config,
        }
    }

    pub async fn run(self: &Arc<Self>) {
        self.ws().await;
        self.wsr().await;
    }
}
