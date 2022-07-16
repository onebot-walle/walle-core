use std::sync::Arc;

use async_trait::async_trait;

use crate::error::WalleResult;
use crate::util::SelfIds;
use crate::EventHandler;
use crate::OneBot;

#[async_trait]
pub trait ActionHandler<E, A, R, const V: u8>: GetStatus + SelfIds + Sync {
    type Config;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH, V>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
    async fn call(&self, action: A) -> WalleResult<R>;
    async fn shutdown(&self) {}
}

pub trait GetStatus {
    fn get_status(&self) -> crate::structs::Status;
}
