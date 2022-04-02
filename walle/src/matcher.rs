use crate::rules::*;
use crate::{Plugin, TempPlugins};
use async_trait::async_trait;
use walle_core::app::StandardArcBot;
use walle_core::{
    BaseEvent, EventContent, Message, MessageContent, Resps, StandardEvent, WalleResult,
};

#[async_trait]
pub trait Matcher: Sync {
    fn _match(&self, _event: &StandardEvent) -> bool {
        println!("default! {:?}", _event);
        true
    }
    async fn handle(&self, session: Session<EventContent>);
    async fn on_startup(&self) {}
    async fn on_shutdown(&self) {}
}

trait SubContent {
    fn pre_parse(session: Session<EventContent>) -> Option<Session<Self>>
    where
        Self: Sized;
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
    pub fn group_id(&self) -> Option<&str> {
        self.event.group_id()
    }

    pub fn user_id(&self) -> &str {
        self.event.user_id()
    }

    pub async fn send(&self, message: Message) -> WalleResult<Resps> {
        if let Some(group_id) = self.group_id() {
            self.bot
                .send_group_message(group_id.to_string(), message)
                .await
        } else {
            self.bot
                .send_private_message(self.user_id().to_string(), message)
                .await
        }
    }

    pub async fn get(&mut self, message: Message, timeout: std::time::Duration) -> WalleResult<()> {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (name, temp) = TempMathcer::new(
            self.user_id().to_string(),
            self.group_id().map(ToString::to_string),
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
    pub user_id: String,
    pub group_id: Option<String>,
    pub tx: tokio::sync::mpsc::Sender<BaseEvent<MessageContent>>,
}

#[async_trait]
impl Matcher for TempMathcer {
    fn _match(&self, event: &StandardEvent) -> bool {
        check_user_id(&event, &self.user_id)
            && self
                .group_id
                .as_ref()
                .map_or(true, |i| check_user_id(&event, &i))
    }
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
    ) -> (String, Plugin) {
        let name = format!("{}-{:?}", user_id, group_id);
        (
            name.clone(),
            Plugin::new(
                name,
                "".to_string(),
                Self {
                    user_id,
                    group_id,
                    tx,
                },
            ),
        )
    }
}
