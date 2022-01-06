use std::sync::Arc;

use crate::action::{Action, Resp};
use crate::app::ArcBot;
use crate::event::Event;
use async_trait::async_trait;

pub(crate) type ArcEventHandler = Arc<dyn EventHandler + Send + Sync + 'static>;
pub(crate) type ArcActionHandler = Arc<dyn ActionHandler + Send + Sync + 'static>;

#[async_trait]
pub trait EventHandler {
    async fn handle(&self, bot: ArcBot, _: Event);
}

#[async_trait]
pub trait ActionHandler {
    async fn handle(&self, action: Action) -> Resp;
}

pub struct DefaultHandler;

#[async_trait]
impl EventHandler for DefaultHandler {
    async fn handle(&self, _: ArcBot, _: Event) {}
}

#[async_trait]
impl ActionHandler for DefaultHandler {
    async fn handle(&self, _: Action) -> Resp {
        Resp::empty_404()
    }
}

pub fn default_handler() -> Arc<DefaultHandler> {
    Arc::new(DefaultHandler)
}
