use crate::{LayeredPreHandler, LayeredRule, Plugin, PreHandler, Rule, TempPlugins};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use walle_core::app::StandardArcBot;
use walle_core::{BaseEvent, EventContent, IntoMessage, MessageContent, Resps, WalleResult};

#[async_trait]
pub trait Handler<C>: Sync {
    fn _match(&self, _session: &Session<C>) -> bool {
        true
    }
    /// if matched will be called before handle, should never fail
    fn _pre_handle(&self, _session: &mut Session<C>) {}
    async fn handle(&self, session: Session<C>);
    // async fn on_startup(&self) {}
    // async fn on_shutdown(&self) {}
}

pub trait HandlerExt<C>: Handler<C> {
    fn rule<R>(self, rule: R) -> LayeredRule<R, Self>
    where
        Self: Sized,
        R: Rule<C>,
    {
        rule.layer(self)
    }
    fn pre_handle<P>(self, pre: P) -> LayeredPreHandler<P, Self>
    where
        Self: Sized,
        P: PreHandler<C>,
    {
        pre.layer(self)
    }
}

impl<C, H: Handler<C>> HandlerExt<C> for H {}

pub struct HandlerFn<I>(I);

pub fn handler_fn<I, C, Fut>(inner: I) -> HandlerFn<I>
where
    I: Fn(Session<C>) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
    C: Sync + Send + 'static,
{
    HandlerFn(inner)
}

impl<C, I, Fut> Handler<C> for HandlerFn<I>
where
    C: Sync + Send + 'static,
    I: Fn(Session<C>) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    fn handle<'a, 'b>(
        &'a self,
        session: Session<C>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'b>>
    where
        'a: 'b,
        Self: 'b,
    {
        Box::pin(async move { self.0(session).await })
    }
}

#[derive(Clone)]
pub struct Session<C> {
    pub bot: walle_core::app::StandardArcBot,
    pub event: walle_core::event::BaseEvent<C>,
    temp_plugins: TempPlugins,
}

impl<C> Session<C> {
    pub fn new(bot: StandardArcBot, event: BaseEvent<C>, temp_plugins: TempPlugins) -> Self {
        Self {
            bot,
            event,
            temp_plugins,
        }
    }

    pub fn replace_evnet(&mut self, event: BaseEvent<C>) {
        self.event = event;
    }
}

impl Session<EventContent> {
    pub fn as_message_session(self) -> Option<Session<MessageContent>> {
        if let Ok(event) = self.event.try_into() {
            Some(Session {
                bot: self.bot,
                event,
                temp_plugins: self.temp_plugins,
            })
        } else {
            None
        }
    }
}

impl Session<MessageContent> {
    pub async fn send<T: IntoMessage>(&self, message: T) -> WalleResult<Resps> {
        if let Some(group_id) = self.event.group_id() {
            self.bot
                .send_group_message(group_id.to_string(), message.into_message())
                .await
        } else {
            self.bot
                .send_private_message(self.event.user_id().to_string(), message.into_message())
                .await
        }
    }

    pub async fn get<T: IntoMessage>(
        &mut self,
        message: T,
        timeout: std::time::Duration,
    ) -> WalleResult<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (name, temp) = TempMathcer::new(
            self.event.user_id().to_string(),
            self.event.group_id().map(ToString::to_string),
            tx,
        );
        self.temp_plugins.lock().await.insert(name.clone(), temp);
        self.send(message).await?;
        match tokio::time::timeout(timeout, rx.recv()).await {
            Ok(Some(event)) => {
                self.replace_evnet(event);
            }
            _ => {
                self.temp_plugins.lock().await.remove(&name);
            }
        };
        Ok(())
    }
}

pub struct TempMathcer {
    pub tx: tokio::sync::mpsc::Sender<BaseEvent<MessageContent>>,
}

#[async_trait]
impl Handler<EventContent> for TempMathcer {
    async fn handle(&self, session: Session<EventContent>) {
        let event = session.event;
        self.tx.send(event.try_into().unwrap()).await.unwrap();
    }
}

impl TempMathcer {
    pub fn new(
        user_id: String,
        group_id: Option<String>,
        tx: tokio::sync::mpsc::Sender<BaseEvent<MessageContent>>,
    ) -> (String, Plugin<EventContent>) {
        use crate::builtin::{group_id_check, user_id_check};
        let name = format!("{}-{:?}", user_id, group_id);
        let matcher = user_id_check(user_id).layer(Self { tx });
        (
            name.clone(),
            if let Some(group_id) = group_id {
                Plugin::new(
                    name,
                    "".to_string(),
                    group_id_check(group_id).layer(matcher),
                )
            } else {
                Plugin::new(name, "".to_string(), matcher)
            },
        )
    }
}
