use crate::{Matcher, Session};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use walle_core::app::StandardArcBot;
use walle_core::{EventContent, EventHandler, Resps, StandardAction, StandardEvent};

pub(crate) type TempPlugins = Arc<Mutex<HashMap<String, Plugin>>>;

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
                if plugin.matcher._match(&session.event) {
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

    #[async_recursion::async_recursion]
    pub async fn handle(&self, session: &Session<EventContent>) {
        for plugin in &self.sub_plugins {
            plugin.handle(session).await;
        }
        if self.matcher._match(&session.event) {
            let matcher = self.matcher.clone();
            let session = session.clone();
            tokio::spawn(async move { matcher.handle(session).await });
        }
    }
}
