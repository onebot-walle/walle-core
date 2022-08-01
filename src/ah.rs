use std::sync::Arc;

use async_trait::async_trait;

use crate::error::WalleResult;
use crate::util::SelfId;
use crate::util::SelfIds;
use crate::EventHandler;
use crate::OneBot;

#[async_trait]
pub trait ActionHandler<E, A, R, const V: u8>: GetStatus + SelfIds + Sync {
    type Config;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
    /// do not use call directly, use OneBot.handle_action instead.
    async fn call(&self, action: A) -> WalleResult<R>;
    async fn before_call_event(&self, event: E) -> WalleResult<E>
    where
        E: Send + 'static,
    {
        Ok(event)
    }
    async fn after_call_event(&self) -> WalleResult<()> {
        Ok(())
    }
    async fn shutdown(&self) {}
}

pub trait GetStatus {
    fn get_status(&self) -> crate::structs::Status;
}

pub struct JoinedHandler<H0, H1>(pub H0, pub H1);

pub trait AHExt<E, A, R, const V: u8> {
    fn join<AH1>(self, action_handler: AH1) -> JoinedHandler<Self, AH1>
    where
        Self: Sized,
    {
        JoinedHandler(self, action_handler)
    }
}

impl<T: ActionHandler<E, A, R, V>, E, A, R, const V: u8> AHExt<E, A, R, V> for T {}

#[async_trait]
impl<H0, H1> SelfIds for JoinedHandler<H0, H1>
where
    H0: SelfIds + Send + Sync + 'static,
    H1: SelfIds + Send + Sync + 'static,
{
    async fn self_ids(&self) -> Vec<String> {
        let mut ids = self.0.self_ids().await;
        ids.extend(self.1.self_ids().await);
        ids
    }
}

impl<H0, H1> GetStatus for JoinedHandler<H0, H1>
//todo
where
    H0: GetStatus,
{
    fn get_status(&self) -> crate::structs::Status {
        self.0.get_status()
    }
}

#[async_trait]
impl<AH0, AH1, E, A, R, const V: u8> ActionHandler<E, A, R, V> for JoinedHandler<AH0, AH1>
where
    AH0: ActionHandler<E, A, R, V> + Send + Sync + 'static,
    AH0::Config: Send + Sync + 'static,
    AH1: ActionHandler<E, A, R, V> + Send + Sync + 'static,
    AH1::Config: Send + Sync + 'static,
    A: SelfId + Send + Sync + 'static,
    R: From<crate::resp::RespError>,
{
    type Config = (AH0::Config, AH1::Config);
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
    async fn call(&self, action: A) -> WalleResult<R> {
        if self.0.self_ids().await.contains(&action.self_id()) {
            self.0.call(action).await
        } else if self.1.self_ids().await.contains(&action.self_id()) {
            self.1.call(action).await
        } else {
            Ok(crate::resp::resp_error::bad_request("bot not exist").into())
        }
    }
    async fn shutdown(&self) {
        self.0.shutdown().await;
        self.1.shutdown().await;
    }
}
