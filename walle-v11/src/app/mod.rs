use crate::action::Resp;
use crate::handler::ArcRespHandler;
use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::RwLock;

mod bot;

pub(crate) type ActionSender = tokio::sync::mpsc::UnboundedSender<crate::action::Action>;
pub(crate) type EchoMap = Arc<RwLock<HashMap<String, tokio::sync::oneshot::Sender<Resp>>>>;

pub struct OneBot {
    pub(crate) handler: ArcRespHandler,
    pub(crate) config: crate::config::AppConfig,
    pub(crate) running: AtomicBool,
    pub(crate) echo_map: EchoMap,
    bots: Arc<RwLock<HashMap<i32, ArcBot>>>,
}

pub struct Bot {
    pub self_id: i32,
    pub action_sender: ActionSender,
    pub echo_map: EchoMap,
}

pub type ArcBot = Arc<Bot>;

impl OneBot {
    pub fn new(handler: ArcRespHandler, config: crate::config::AppConfig) -> Self {
        Self {
            handler,
            config,
            running: AtomicBool::new(false),
            echo_map: Arc::default(),
            bots: Arc::default(),
        }
    }

    pub async fn get_bot(&self, bot_id: i32) -> Option<ArcBot> {
        self.bots.read().await.get(&bot_id).map(|bot| bot.clone())
    }

    pub async fn add_bot(&self, self_id: i32, action_sender: ActionSender) -> ArcBot {
        let bot = Arc::new(Bot {
            self_id,
            action_sender,
            echo_map: self.echo_map.clone(),
        });
        self.bots.write().await.insert(self_id, bot.clone());
        bot
    }

    pub async fn remove_bot(&self, bot_id: i32) {
        self.bots.write().await.remove(&bot_id);
    }

    pub async fn run(self: &Arc<Self>) {
        self.ws().await;
        self.wsr().await;
    }
}
