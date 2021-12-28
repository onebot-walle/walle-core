use crate::handler::ArcEventHandler;
use std::collections::HashMap;
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::RwLock;

mod bot;

pub use bot::{ArcBot, Bot};

pub(crate) type RespSender = tokio::sync::oneshot::Sender<crate::action::Resp>;
pub(crate) type ActionSender =
    tokio::sync::mpsc::UnboundedSender<(crate::action::Action, RespSender)>;

pub struct OneBot {
    pub(crate) handler: ArcEventHandler,
    pub(crate) config: walle_core::AppConfig,
    pub(crate) running: AtomicBool,
    bots: Arc<RwLock<HashMap<i32, ArcBot>>>,
}

impl OneBot {
    pub fn new(handler: ArcEventHandler, config: walle_core::AppConfig) -> Self {
        Self {
            handler,
            config,
            running: AtomicBool::new(false),
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
