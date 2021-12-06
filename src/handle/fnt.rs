use async_trait::async_trait;
use futures_util::future::BoxFuture;
use serde::{de::DeserializeOwned, Serialize};

#[cfg(feature = "app")]
use crate::app::ArcBot;

#[async_trait]
impl<A, R, F> super::ActionHandler<A, R> for F
where
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
    F: Fn(A) -> BoxFuture<'static, R> + Sync + Send + 'static,
{
    async fn handle(&self, action: A) -> R {
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
