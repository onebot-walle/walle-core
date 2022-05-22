use std::sync::Arc;

use async_trait::async_trait;

#[async_trait]
pub trait WsHooks<T>: Sync + Send {
    /// only available on server side
    async fn before_server_start(&self, _: &Arc<T>) {}
    /// only available on client side
    async fn before_client_connect(&self, _: &Arc<T>) {}
    /// only available on client side
    async fn before_client_reconnect(&self, _: &Arc<T>) {}
    async fn on_connect(&self, _: &Arc<T>) {}
    async fn on_disconnect(&self, _: &Arc<T>) {}
    async fn on_shutdown(&self, _: &Arc<T>) {}
}

pub type BoxWsHooks<T> = Box<dyn WsHooks<T>>;

/// default empty ws hooks
pub(crate) struct EmptyWsHooks<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> EmptyWsHooks<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

#[async_trait]
impl<T: Send + Sync> WsHooks<T> for EmptyWsHooks<T> {}

/// default empty ws hooks
pub(crate) fn empty_ws_hooks<T: Send + Sync + 'static>() -> BoxWsHooks<T> {
    Box::new(EmptyWsHooks::new())
}
