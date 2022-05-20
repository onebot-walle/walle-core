use std::sync::Arc;

use async_trait::async_trait;
use futures_util::future::BoxFuture;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "app")]
use crate::app::ArcBot;

#[async_trait]
impl<A, R, ER, F, OB> super::ActionHandler<A, R, OB> for Arc<F>
where
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + From<ER> + std::fmt::Debug + Send + 'static,
    F: Fn(A) -> BoxFuture<'static, Result<R, ER>> + Send + Sync + 'static,
    OB: Sync,
{
    type Error = ER;
    async fn handle(&self, action: A, _ob: &OB) -> Result<R, ER> {
        self(action).await
    }
}

#[cfg(feature = "app")]
#[async_trait]
impl<E, F, A, R> super::EventHandler<E, A, R> for F
where
    E: Clone + DeserializeOwned + std::fmt::Debug + Send + 'static,
    F: Fn(E) -> BoxFuture<'static, ()> + Sync + Send + 'static,
{
    async fn handle(&self, _: ArcBot<A, R>, event: E) {
        self(event).await
    }
}
