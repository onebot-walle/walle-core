use crate::prelude::*;
use lru::LruCache;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use walle_core::WalleResult;

pub struct UnionBot {
    self_id: String,
    inner: RwLock<LruCache<String, Bot>>,
    event_tx: tokio::sync::mpsc::UnboundedSender<v12Event>,
    event_rx: tokio::sync::mpsc::UnboundedReceiver<v12Event>,
    group_map: RwLock<HashMap<String, Vec<String>>>,
    friend_map: RwLock<HashMap<String, Vec<String>>>,
    config: UnionConfig,
}

impl UnionBot {
    pub fn new(config: UnionConfig) -> Self {
        let (event_tx, event_rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            self_id: config.mode.bot_id(),
            inner: RwLock::new(LruCache::unbounded()),
            event_tx,
            event_rx,
            group_map: RwLock::default(),
            friend_map: RwLock::default(),
            config,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UnionConfig {
    pub apps: walle_core::AppConfig,
    pub impls: walle_core::ImplConfig,
    #[serde(default)]
    pub mode: UnionMode,
}

impl UnionConfig {
    pub fn load_or_new(path: &str) -> Self {
        use std::io::Read;
        let mut file = std::fs::File::open(path).unwrap();
        let mut content = String::new();
        file.read_to_string(&mut content).unwrap();
        let config: UnionConfig = toml::from_str(&content).unwrap();
        config
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UnionMode {
    None,                     // 仅作为转发节点
    Union { bot_id: String }, // 合并为单一 Bot (以下模式均为 Unioned)
    Round { bot_id: String }, // 轮询均衡负载
    Main { bot_id: String, main: String }, // 主副节点
                              // WRR(f32),  // 加权轮询均衡 ToDo
}

impl Default for UnionMode {
    fn default() -> Self {
        UnionMode::Union {
            bot_id: "walle-hub".to_string(),
        }
    }
}

impl UnionMode {
    pub(crate) fn bot_id(&self) -> String {
        match self {
            UnionMode::Union { bot_id } => bot_id.to_owned(),
            UnionMode::Round { bot_id } => bot_id.to_owned(),
            UnionMode::Main { bot_id, .. } => bot_id.to_owned(),
            UnionMode::None => "".to_owned(),
        }
    }
}

#[derive(Clone)]
pub enum Bot {
    V11(v11Bot),
    V12(v12Bot<v12Action, v12RespContent>),
}

impl Bot {
    async fn call_action_resp(&self, action: v12Action) -> WalleResult<v12Resp> {
        match self {
            Bot::V12(bot) => bot.call_action_resp(action).await,
            Bot::V11(bot) => todo!(),
            // bot.call_action(action).await.map(|r| r.try_into()?),
        }
    }
}

impl UnionBot {
    pub(crate) async fn run(&self) {
        match &self.config.mode {
            UnionMode::None => self.just_redirect().await,
            UnionMode::Union { .. } => self.union().await,
            UnionMode::Round { .. } => self.round().await,
            UnionMode::Main { main, .. } => self.main(main).await,
        }
    }
    async fn just_redirect(&self) {}
    async fn union(&self) {}
    async fn round(&self) {}
    async fn main(&self, _: &str) {}
}
