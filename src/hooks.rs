use std::sync::Arc;

use async_trait::async_trait;

use crate::{impls::CustomOneBot, Action, RespContent, Event};

#[async_trait]
pub trait ImplHooks<E, A, R>: Sync + Send {
    async fn on_ws_connect(&self, _: Arc<CustomOneBot<E, A, R>>) {}
    async fn on_ws_disconnect(&self, _: Arc<CustomOneBot<E, A, R>>) {}
}

pub type ArcImplHooks<E, A, R> = Arc<dyn ImplHooks<E, A, R>>;

#[async_trait]
pub trait AppHooks: Sync {
    async fn on_ws_connect() {}
    async fn on_ws_disconnect() {}
}

pub struct DefaultHooks;
impl ImplHooks<Event, Action, RespContent> for DefaultHooks {}
impl DefaultHooks {
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }
}
