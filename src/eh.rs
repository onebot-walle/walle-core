use std::future::Future;
use std::sync::Arc;

use crate::action::Action;
use crate::error::WalleResult;
use crate::event::Event;
use crate::resp::Resp;
use crate::ActionHandler;
use crate::OneBot;

/// EventHandler 接收 Event, 产生 Action
///
/// 对于应用端，EventHandler 为具体实现
///
/// 对于协议端，EventHandler 为 OBC
pub trait EventHandler<E = Event, A = Action, R = Resp>: Sync {
    type Config;
    fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Self::Config,
    ) -> impl Future<Output = WalleResult<Vec<tokio::task::JoinHandle<()>>>>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static;
    /// do not use directly, use OneBot.handle_event instead.
    fn call<AH, EH>(
        &self,
        event: E,
        ob: &Arc<OneBot<AH, EH>>,
    ) -> impl Future<Output = WalleResult<()>> + Send
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static;
    fn before_call_action<AH, EH>(
        &self,
        action: A,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> impl Future<Output = WalleResult<A>> + Send
    where
        A: Send + 'static,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        async { Ok(action) }
    }
    fn after_call_action<AH, EH>(
        &self,
        resp: R,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> impl Future<Output = WalleResult<R>> + Send
    where
        R: Send + 'static,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        async { Ok(resp) }
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

use crate::ah::JoinedHandler;

pub trait EHExt<E, A, R> {
    fn join<EH1>(self, event_handler: EH1) -> JoinedHandler<Self, EH1>
    where
        Self: Sized,
    {
        JoinedHandler(self, event_handler)
    }
}

impl<T: EventHandler<E, A, R>, E, A, R> EHExt<E, A, R> for T {}

impl<EH0, EH1, E, A, R> EventHandler<E, A, R> for JoinedHandler<EH0, EH1>
where
    EH0: EventHandler<E, A, R> + Send + Sync + 'static,
    EH0::Config: Send + Sync + 'static,
    EH1: EventHandler<E, A, R> + Send + Sync + 'static,
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
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        let mut joins = self.0.start(ob, config.0).await?;
        joins.extend(self.1.start(ob, config.1).await?.into_iter());
        Ok(joins)
    }
    async fn call<AH, EH>(&self, event: E, ob: &Arc<OneBot<AH, EH>>) -> WalleResult<()>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        self.0.call(event.clone(), ob).await?;
        self.1.call(event, ob).await
    }
    async fn shutdown(&self) {
        self.0.shutdown().await;
        self.1.shutdown().await;
    }
}
