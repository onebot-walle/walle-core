use std::sync::Arc;

use async_trait::async_trait;

use crate::error::WalleResult;
use crate::ActionHandler;
use crate::OneBot;

#[async_trait]
pub trait EventHandler<E, A, R, const V: u8>: Sync {
    type Config;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
    /// do not use directly, use OneBot.handle_event instead.
    async fn call(&self, event: E) -> WalleResult<()>;
    async fn before_call_action(&self, action: A) -> WalleResult<A>
    where
        A: Send + 'static,
    {
        Ok(action)
    }
    async fn after_call_action(&self, resp: R) -> WalleResult<R>
    where
        R: Send + 'static,
    {
        Ok(resp)
    }
    async fn shutdown(&self) {}
}

use crate::ah::JoinedHandler;

pub trait EHExt<E, A, R, const V: u8> {
    fn join<EH1>(self, event_handler: EH1) -> JoinedHandler<Self, EH1>
    where
        Self: Sized,
    {
        JoinedHandler(self, event_handler)
    }
}

impl<T: EventHandler<E, A, R, V>, E, A, R, const V: u8> EHExt<E, A, R, V> for T {}

#[async_trait]
impl<EH0, EH1, E, A, R, const V: u8> EventHandler<E, A, R, V> for JoinedHandler<EH0, EH1>
where
    EH0: EventHandler<E, A, R, V> + Send + Sync + 'static,
    EH0::Config: Send + Sync + 'static,
    EH1: EventHandler<E, A, R, V> + Send + Sync + 'static,
    EH1::Config: Send + Sync + 'static,
    E: Clone + Send + Sync + 'static,
{
    type Config = (EH0::Config, EH1::Config);
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static,
    {
        let mut joins = self.0.start(ob, config.0).await?;
        joins.extend(self.1.start(ob, config.1).await?.into_iter());
        Ok(joins)
    }
    async fn call(&self, event: E) -> WalleResult<()> {
        self.0.call(event.clone()).await?;
        self.1.call(event).await
    }
    async fn shutdown(&self) {
        self.0.shutdown().await;
        self.1.shutdown().await;
    }
}
