use std::sync::Arc;

use async_trait::async_trait;

use crate::error::WalleResult;
use crate::util::SelfIds;
use crate::EventHandler;
use crate::OneBot;

#[async_trait]
pub trait ActionHandler<E, A, R, const V: u8>: GetStatus + SelfIds {
    type Config;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH, V>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
    async fn call<AH, EH>(&self, action: A, ob: &Arc<OneBot<AH, EH, V>>) -> WalleResult<R>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
}

pub trait GetStatus {
    fn get_status(&self) -> crate::structs::Status;
}
