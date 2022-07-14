use std::sync::Arc;

use async_trait::async_trait;

use crate::error::WalleResult;
use crate::ActionHandler;
use crate::OneBot;

#[async_trait]
pub trait EventHandler<E, A, R, const V: u8> {
    type Config;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH, V>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
    async fn call<AH, EH>(&self, event: E, ob: &Arc<OneBot<AH, EH, V>>)
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
}
