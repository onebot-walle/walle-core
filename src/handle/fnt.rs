use async_trait::async_trait;
use futures_util::future::BoxFuture;
use serde::{de::DeserializeOwned, Serialize};

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

#[async_trait]
impl<E, F> super::EventHandler<E> for F
where
    E: Clone + DeserializeOwned + std::fmt::Debug + Send + 'static,
    F: Fn(E) -> BoxFuture<'static, ()> + Sync + Send + 'static,
{
    async fn handle(&self, event: E) {
        self(event).await
    }
}

