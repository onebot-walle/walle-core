use crate::{Plugin, TempPlugins};
use async_trait::async_trait;
use std::future::Future;
use std::pin::Pin;
use walle_core::app::StandardArcBot;
use walle_core::{BaseEvent, EventContent, Message, MessageContent, Resps, WalleResult};

#[async_trait]
pub trait Handler<C>: Sync {
    fn _match(&self, _bot: &StandardArcBot, _event: &BaseEvent<C>) -> bool {
        true
    }
    async fn handle(&self, session: Session<C>);
    // async fn on_startup(&self) {}
    // async fn on_shutdown(&self) {}
}

impl<C, T, Fut> Handler<C> for T
where
    C: Sync + Send + 'static,
    T: Fn(Session<C>) -> Fut + Send + Sync,
    Fut: Future<Output = ()> + Send,
{
    fn handle<'a, 'async_trait>(
        &'a self,
        session: Session<C>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'async_trait>>
    where
        'a: 'async_trait,
        Self: 'async_trait,
    {
        Box::pin(async move {
            self(session).await;
        })
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
    pub async fn send(&self, message: Message) -> WalleResult<Resps> {
        if let Some(group_id) = self.event.group_id() {
            self.bot
                .send_group_message(group_id.to_string(), message)
                .await
        } else {
            self.bot
                .send_private_message(self.event.user_id().to_string(), message)
                .await
        }
    }

    pub async fn get(&mut self, message: Message, timeout: std::time::Duration) -> WalleResult<()> {
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
        use crate::builtin::{group_id_layer, user_id_layer};
        let name = format!("{}-{:?}", user_id, group_id);
        let matcher = user_id_layer(user_id, Self { tx });
        (
            name.clone(),
            if let Some(group_id) = group_id {
                Plugin::new(name, "".to_string(), group_id_layer(group_id, matcher))
            } else {
                Plugin::new(name, "".to_string(), matcher)
            },
        )
    }
}
