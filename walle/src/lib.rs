use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rules::check_user_id;
use tokio::sync::Mutex;
use walle_core::{
    app::StandardArcBot, AppConfig, BaseEvent, EventContent, EventHandler, Message, MessageContent,
    Resps, StandardAction, StandardEvent, WalleResult,
};

pub mod plugins;
pub mod rules;

pub struct Walle {
    pub ob: walle_core::app::StandardOneBot,
}

type TempPlugins = Arc<Mutex<HashMap<String, Plugin>>>;

#[derive(Default)]
pub struct Plugins(pub Vec<Plugin>, TempPlugins);

impl From<Vec<Plugin>> for Plugins {
    fn from(plugins: Vec<Plugin>) -> Self {
        Plugins(plugins, Arc::new(Mutex::new(HashMap::new())))
    }
}

#[async_trait]
impl EventHandler<StandardEvent, StandardAction, Resps> for Plugins {
    async fn handle(&self, bot: StandardArcBot, event: StandardEvent) {
        let session = Session::new(bot, event, self.1.clone());
        if let Some(p) = {
            let mut temp_plugins = self.1.lock().await;
            let mut found: Option<String> = None;
            for (k, plugin) in temp_plugins.iter() {
                if plugin.matcher._match(&session.event).await {
                    found = Some(k.clone());
                    break;
                }
            }
            found.and_then(|i| temp_plugins.remove(&i))
        } {
            p.matcher.handle(session).await;
            return;
        }
        for plugin in &self.0 {
            plugin.handle(&session).await;
        }
    }
}

impl Walle {
    pub fn new(config: AppConfig, plugins: Plugins) -> Self {
        Self {
            ob: walle_core::app::StandardOneBot::new(config, Box::new(plugins)),
        }
    }
}

#[derive(Clone)]
pub struct Plugin {
    pub name: String,
    pub description: String,
    pub matcher: Arc<dyn Matcher + Sync + Send + 'static>,
    sub_plugins: Vec<Plugin>,
}

impl Plugin {
    pub fn new<T0: ToString, T1: ToString>(
        name: T0,
        description: T1,
        matcher: impl Matcher + Sync + Send + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            matcher: Arc::new(matcher),
            sub_plugins: vec![],
        }
    }

    pub fn sub_plugin(&mut self, plugin: Plugin) {
        self.sub_plugins.push(plugin);
    }

    pub async fn handle(&self, session: &Session<EventContent>) {
        for plugin in &self.sub_plugins {
            let matcher = plugin.matcher.clone();
            let session = session.clone();
            tokio::spawn(async move { matcher.handle(session).await });
        }
        let matcher = self.matcher.clone();
        let session = session.clone();
        tokio::spawn(async move { matcher.handle(session).await });
    }
}

#[async_trait]
pub trait Matcher: Sync {
    async fn _match(&self, _event: &StandardEvent) -> bool {
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

    pub async fn get(&mut self, _message: Message, timeout: std::time::Duration) {
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);
        let (name, temp) = TempMathcer::new(
            self.user_id().to_string(),
            self.group_id().map(ToString::to_string),
            tx,
        );
        self.temp_plugins.lock().await.insert(name.clone(), temp);
        match tokio::time::timeout(timeout, rx.recv()).await {
            Ok(Some(event)) => {
                self.replace_evnet(event);
            }
            _ => {
                self.temp_plugins.lock().await.remove(&name);
            }
        };
    }
}

pub struct TempMathcer {
    pub user_id: String,
    pub group_id: Option<String>,
    pub tx: tokio::sync::mpsc::Sender<BaseEvent<MessageContent>>,
}

#[async_trait]
impl Matcher for TempMathcer {
    async fn _match(&self, event: &StandardEvent) -> bool {
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
