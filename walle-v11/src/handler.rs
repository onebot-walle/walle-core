use std::sync::Arc;

use crate::action::{Action, Resp};
use crate::app::{ArcBot, OneBot};
use crate::event::Event;
use async_trait::async_trait;

pub(crate) type ArcRespHandler = Arc<dyn RespHandler + Send + Sync + 'static>;
pub(crate) type ArcActionHandler = Arc<dyn ActionHandler + Send + Sync + 'static>;

#[async_trait]
pub trait EventHandler {
    async fn handle(&self, bot: ArcBot, _: Event);
}

#[async_trait]
pub trait RespHandler: EventHandler {
    async fn handle_resp(&self, ob: Arc<OneBot>, _: Resp);
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
impl RespHandler for DefaultHandler {
    async fn handle_resp(&self, _: Arc<OneBot>, _: Resp) {}
}
#[async_trait]
impl ActionHandler for DefaultHandler {
    async fn handle(&self, action: Action) -> Resp {
        Resp::empty_404(action.echo)
    }
}
pub fn default_handler() -> Arc<DefaultHandler> {
    Arc::new(DefaultHandler)
}
