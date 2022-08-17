use std::sync::Arc;

use async_trait::async_trait;

use crate::error::WalleResult;
use crate::structs::{Bot, Selft, Status};
use crate::util::GetSelf;
use crate::EventHandler;
use crate::OneBot;

#[async_trait]
pub trait ActionHandler<E, A, R>: GetStatus + Sync {
    type Config;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static;
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

#[async_trait]
pub trait GetSelfs {
    async fn get_selfs(&self) -> Vec<Selft>;
    async fn get_impl(&self, selft: &Selft) -> String;
}

#[async_trait]
pub trait GetStatus: GetSelfs {
    async fn is_good(&self) -> bool;
    async fn get_status(&self) -> Status
    where
        Self: Sized,
    {
        Status {
            good: self.is_good().await,
            bots: self
                .get_selfs()
                .await
                .into_iter()
                .map(|selft| Bot {
                    selft,
                    online: true,
                })
                .collect(),
        }
    }
}

pub struct JoinedHandler<H0, H1>(pub H0, pub H1);

pub trait AHExt<E, A, R> {
    fn join<AH1>(self, action_handler: AH1) -> JoinedHandler<Self, AH1>
    where
        Self: Sized,
    {
        JoinedHandler(self, action_handler)
    }
}

impl<T: ActionHandler<E, A, R>, E, A, R> AHExt<E, A, R> for T {}

#[async_trait]
impl<H0, H1> GetSelfs for JoinedHandler<H0, H1>
//todo
where
    H0: GetSelfs + Send + Sync,
    H1: GetSelfs + Send + Sync,
{
    async fn get_selfs(&self) -> Vec<crate::structs::Selft> {
        let mut r = self.0.get_selfs().await;
        r.extend(self.1.get_selfs().await.into_iter());
        r
    }
    async fn get_impl(&self, selft: &Selft) -> String {
        if self.0.get_selfs().await.contains(selft) {
            self.0.get_impl(selft).await
        } else {
            self.1.get_impl(selft).await
        }
    }
}

#[async_trait]
impl<H0, H1> GetStatus for JoinedHandler<H0, H1>
//todo
where
    H0: GetStatus + Send + Sync,
    H1: GetStatus + Send + Sync,
{
    async fn is_good(&self) -> bool {
        self.0.is_good().await && self.1.is_good().await
    }
}

#[async_trait]
impl<AH0, AH1, E, A, R> ActionHandler<E, A, R> for JoinedHandler<AH0, AH1>
where
    AH0: ActionHandler<E, A, R> + Send + Sync + 'static,
    AH0::Config: Send + Sync + 'static,
    AH1: ActionHandler<E, A, R> + Send + Sync + 'static,
    AH1::Config: Send + Sync + 'static,
    A: GetSelf + Send + Sync + 'static,
    R: From<crate::resp::RespError>,
{
    type Config = (AH0::Config, AH1::Config);
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        let mut joins = self.0.start(ob, config.0).await?;
        joins.extend(self.1.start(ob, config.1).await?.into_iter());
        Ok(joins)
    }
    async fn call(&self, action: A) -> WalleResult<R> {
        if self.0.get_selfs().await.contains(&action.get_self()) {
            self.0.call(action).await
        } else if self.1.get_selfs().await.contains(&action.get_self()) {
            self.1.call(action).await
        } else {
            Ok(crate::resp::resp_error::who_am_i("").into())
        }
    }
    async fn shutdown(&self) {
        self.0.shutdown().await;
        self.1.shutdown().await;
    }
}
