use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use rules::check_user_id;
use tokio::sync::Mutex;
use walle_core::{
    app::StandardArcBot, AppConfig, EventHandler, Message, Resps, StandardAction, StandardEvent,
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
                if plugin.matcher._match(&session).await {
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
            let matcher = plugin.matcher.clone();
            let session = session.clone();
            tokio::spawn(async move { matcher.handle(session).await });
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
}

impl Plugin {
    fn new<T0: ToString, T1: ToString>(
        name: T0,
        description: T1,
        matcher: impl Matcher + Sync + Send + 'static,
    ) -> Self {
        Self {
            name: name.to_string(),
            description: description.to_string(),
            matcher: Arc::new(matcher),
        }
    }
}

#[async_trait]
pub trait Matcher: Sync {
    async fn _match(&self, _session: &Session) -> bool {
        true
    }
    async fn handle(&self, session: Session);
    async fn on_startup(&self)
    where
        Self: Sized,
    {
    }
    async fn on_shutdown(&self)
    where
        Self: Sized,
    {
    }
}

#[derive(Clone)]
pub struct Session {
    pub bot: walle_core::app::StandardArcBot,
    pub event: walle_core::event::StandardEvent,
    temp_plugins: TempPlugins,
}

impl Session {
    pub fn new(bot: StandardArcBot, event: StandardEvent, temp_plugins: TempPlugins) -> Self {
        Self {
            bot,
            event,
            temp_plugins,
        }
    }

    pub async fn send(&self, _message: Message) {
        //todo
    }

    pub async fn get(&mut self, _message: Message) {
        // let temp = TempMathcer::new(user_id, group_id, tx);
    }
}

pub struct TempMathcer {
    pub user_id: String,
    pub group_id: Option<String>,
    pub tx: tokio::sync::mpsc::Sender<StandardEvent>,
}

#[async_trait]
impl Matcher for TempMathcer {
    async fn _match(&self, session: &Session) -> bool {
        check_user_id(&session.event, &self.user_id)
            && self
                .group_id
                .as_ref()
                .map_or(true, |i| check_user_id(&session.event, &i))
    }
    async fn handle(&self, session: Session) {
        self.tx.send(session.event).await.unwrap();
    }
}

impl TempMathcer {
    pub fn new(
        user_id: String,
        group_id: Option<String>,
        tx: tokio::sync::mpsc::Sender<StandardEvent>,
    ) -> Plugin {
        Plugin {
            name: format!("{}-{:?}", user_id, group_id),
            description: "".to_string(),
            matcher: Arc::new(Self {
                user_id,
                group_id,
                tx,
            }),
        }
    }
}
