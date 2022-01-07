use crate::{event::EventContent, Event};
use async_trait::async_trait;
use colored::*;
use std::sync::Arc;
use tracing::{debug, info};
use walle_core::{app::ArcBot, EventHandler};

pub struct DefaultHandler;

impl DefaultHandler {
    pub fn arc() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait]
impl<A, R> EventHandler<Event, A, R> for DefaultHandler
where
    A: Send + 'static,
    R: Send + 'static,
{
    async fn handle(&self, _: ArcBot<A, R>, event: Event) {
        match event.content {
            EventContent::Message(msg_c) => {
                info!(target: "Walle-core", "[{}] Message -> from {}: {}", event.self_id.to_string().red(), msg_c.user_id.to_string().blue(), msg_c.raw_message.green());
            }
            EventContent::MetaEvent(meta_c) => {
                debug!(target: "Walle-core", "[{}] Meta -> Type {}", event.self_id.to_string().red(), meta_c.detail_type().green());
            }
            _ => {}
        }
    }
}
