use std::future::Future;
use std::sync::Arc;

use crate::action::Action;
use crate::error::WalleResult;
use crate::event::Event;
use crate::prelude::Version;
use crate::resp::Resp;
use crate::structs::{Bot, Selft, Status};
use crate::util::GetSelf;
use crate::EventHandler;
use crate::OneBot;

/// ActionHandler 接收 Action, 产生 Event
///
/// 对于应用端，ActionHandler 为 OBC
///
/// 对于协议端，ActionHandler 为具体实现
pub trait ActionHandler<E = Event, A = Action, R = Resp>: GetStatus + GetVersion {
    type Config;
    fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Self::Config,
    ) -> impl Future<Output = WalleResult<Vec<tokio::task::JoinHandle<()>>>> + Send
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static;
    /// do not use call directly, use OneBot.handle_action instead.
    fn call<AH, EH>(
        &self,
        action: A,
        ob: &Arc<OneBot<AH, EH>>,
    ) -> impl Future<Output = WalleResult<R>> + Send
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static;
    fn before_call_event<AH, EH>(
        &self,
        event: E,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> impl Future<Output = WalleResult<E>> + Send
    where
        E: Send + 'static,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        async { Ok(event) }
    }
    fn after_call_event<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> impl Future<Output = WalleResult<()>> + Send
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        async { Ok(()) }
    }
    fn shutdown(&self) -> impl Future<Output = ()> {
        async {}
    }
    fn on_onebot_connect<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> impl Future<Output = WalleResult<()>>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        async { Ok(()) }
    }
    fn on_onebot_disconnect<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> impl Future<Output = WalleResult<()>>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        async { Ok(()) }
    }
}

/// supertrait for ActionHandler
pub trait GetSelfs {
    fn get_selfs(&self) -> impl Future<Output = Vec<Selft>> + Send;
    fn get_impl(&self, selft: &Selft) -> impl Future<Output = String> + Send;
}

impl<T: GetSelfs + Send + Sync> GetSelfs for Arc<T> {
    async fn get_impl(&self, selft: &Selft) -> String {
        self.as_ref().get_impl(selft).await
    }
    async fn get_selfs(&self) -> Vec<Selft> {
        self.as_ref().get_selfs().await
    }
}

/// supertrait for ActionHandler
pub trait GetStatus: GetSelfs + Sync {
    fn is_good(&self) -> impl Future<Output = bool> + Send;
    fn get_status(&self) -> impl Future<Output = Status> + Send
    where
        Self: Sized,
    {
        async {
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
}

pub trait GetVersion {
    fn get_version(&self) -> Version;
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

impl<H0, H1> GetVersion for JoinedHandler<H0, H1>
where
    H0: GetVersion,
{
    fn get_version(&self) -> Version {
        self.0.get_version()
    }
}

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
    async fn call<AH, EH>(&self, action: A, ob: &Arc<OneBot<AH, EH>>) -> WalleResult<R>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        if self.0.get_selfs().await.contains(&action.get_self()) {
            self.0.call(action, ob).await
        } else if self.1.get_selfs().await.contains(&action.get_self()) {
            self.1.call(action, ob).await
        } else {
            Ok(crate::resp::resp_error::who_am_i("").into())
        }
    }
    async fn shutdown(&self) {
        self.0.shutdown().await;
        self.1.shutdown().await;
    }
}
