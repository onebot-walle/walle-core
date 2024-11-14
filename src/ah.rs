use std::future::Future;
use std::sync::Arc;

use crate::action::Action;
use crate::error::WalleResult;
use crate::event::Event;
use crate::resp::Resp;
use crate::structs::Selft;
use crate::structs::Status;
use crate::util::GetSelf;
use crate::EventHandler;
use crate::OneBot;

/// ActionHandler 接收 Action, 产生 Event
///
/// 对于应用端，ActionHandler 为 OBC
///
/// 对于协议端，ActionHandler 为具体实现
pub trait ActionHandler<E = Event, A = Action, R = Resp>: GenStatus {
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

pub trait GenStatus {
    fn gen_status(&self) -> Status;
    fn contains_bot(&self, bot: &Selft) -> bool;
}

impl<AH: GenStatus, EH> GenStatus for OneBot<AH, EH> {
    fn gen_status(&self) -> Status {
        self.action_handler.gen_status()
    }
    fn contains_bot(&self, bot: &Selft) -> bool {
        self.action_handler.contains_bot(bot)
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

impl<AH0, AH1> GenStatus for JoinedHandler<AH0, AH1>
where
    AH0: GenStatus,
    AH1: GenStatus,
{
    fn contains_bot(&self, bot: &Selft) -> bool {
        self.0.contains_bot(bot) || self.1.contains_bot(bot)
    }
    fn gen_status(&self) -> Status {
        let status0 = self.0.gen_status();
        let status1 = self.1.gen_status();
        Status {
            good: status0.good && status1.good,
            bots: status0.bots.into_iter().chain(status1.bots).collect(),
        }
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
        if self.0.contains_bot(&action.get_self()) {
            self.0.call(action, ob).await
        } else if self.1.contains_bot(&action.get_self()) {
            self.1.call(action, ob).await
        } else {
            Ok(crate::resp::resp_error::who_am_i("").into())
        }
    }
    async fn before_call_event<AH, EH>(&self, event: E, ob: &Arc<OneBot<AH, EH>>) -> WalleResult<E>
    where
        E: Send + 'static,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        let event = self.0.before_call_event(event, ob).await?;
        self.1.before_call_event(event, ob).await
    }
    async fn after_call_event<AH, EH>(&self, ob: &Arc<OneBot<AH, EH>>) -> WalleResult<()>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        self.0.after_call_event(ob).await?;
        self.1.after_call_event(ob).await
    }
    async fn shutdown(&self) {
        self.0.shutdown().await;
        self.1.shutdown().await
    }
    async fn on_onebot_connect<AH, EH>(&self, ob: &Arc<OneBot<AH, EH>>) -> WalleResult<()>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        self.0.on_onebot_connect(ob).await?;
        self.1.on_onebot_connect(ob).await
    }
    async fn on_onebot_disconnect<AH, EH>(&self, ob: &Arc<OneBot<AH, EH>>) -> WalleResult<()>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        self.0.on_onebot_disconnect(ob).await?;
        self.1.on_onebot_disconnect(ob).await
    }
}
