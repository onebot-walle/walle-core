use serde::{de::DeserializeOwned, Serialize};
use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        atomic::{AtomicU8, Ordering},
        Arc,
    },
};
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    config::AppConfig, event::BaseEvent, Action, ActionResp, ActionRespContent, EventContent,
    FromStandard, WalleError, WalleResult, RUNNING, SHUTDOWN,
};

mod bot;

pub(crate) type ArcEventHandler<E, A, R> =
    Arc<dyn crate::handle::EventHandler<BaseEvent<E>, A, R> + Send + Sync>;
pub(crate) type CustomRespSender<R> = tokio::sync::oneshot::Sender<ActionResp<R>>;
pub(crate) type CustomActionSender<A, R> =
    tokio::sync::mpsc::UnboundedSender<(A, CustomRespSender<R>)>;

/// OneBot v12 无扩展应用端实例
pub type OneBot = CustomOneBot<EventContent, Action, ActionRespContent>;

/// OneBot Application 实例
///
/// E: EventContent 可以参考 crate::evnt::EventContent
/// A: Action 可以参考 crate::action::Action
/// R: ActionResp 可以参考 crate::action_resp::ActionResps
///
/// 如果希望包含 OneBot 的标准内容，可以使用 untagged enum 包裹。
///
/// 不同于实现端，应用端同时只能启用一种通讯协议，当存在多个通讯设定时，
/// 其优先级顺序如下：
///
/// Http( 未实现 ) >> HttpWebhook( 未实现 ) >> 正向 WebSocket >> 反向 WebSocket
pub struct CustomOneBot<E, A, R> {
    pub config: AppConfig,
    pub(crate) event_handler: ArcEventHandler<E, A, R>,
    pub(crate) status: AtomicU8,
    pub bots: RwLock<HashMap<String, ArcBot<A, R>>>,
}

pub type ArcBot<A, R> = Arc<Bot<A, R>>;

pub struct Bot<A, R> {
    #[allow(dead_code)]
    self_id: String,
    sender: CustomActionSender<A, R>,
}

impl<E, A, R> CustomOneBot<E, A, R>
where
    E: Clone + DeserializeOwned + Send + 'static + Debug,
    A: FromStandard<Action> + Clone + Serialize + Send + 'static + Debug,
    R: Clone + DeserializeOwned + Send + 'static + Debug,
{
    pub fn new(config: AppConfig, event_handler: ArcEventHandler<E, A, R>) -> Self {
        Self {
            config,
            event_handler,
            status: AtomicU8::default(),
            bots: RwLock::default(),
        }
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub async fn get_bot(&self, bot_id: &str) -> Option<ArcBot<A, R>> {
        self.bots.read().await.get(bot_id).map(|bot| bot.clone())
    }

    pub async fn get_bots(&self) -> HashMap<String, ArcBot<A, R>> {
        self.bots.read().await.clone()
    }

    pub(crate) async fn insert_bot(
        &self,
        bot_id: &str,
        sender: &CustomActionSender<A, R>,
    ) -> ArcBot<A, R> {
        let bot = Arc::new(Bot::new(bot_id.to_string(), sender.clone()));
        self.bots
            .write()
            .await
            .insert(bot_id.to_string(), bot.clone());
        bot
    }

    pub(crate) async fn remove_bot(&self, bot_id: &str) -> Option<ArcBot<A, R>> {
        self.bots.write().await.remove(bot_id)
    }

    /// 运行 OneBot 实例
    ///
    /// 请注意该方法仅新建协程运行网络通讯协议，本身并不阻塞
    ///
    /// 当重复运行同一个实例或未设置任何通讯协议，将会返回 Err
    ///
    /// 请确保在弃用 bot 前调用 shutdown，否则无法 drop。
    pub async fn run(self: &Arc<Self>) -> WalleResult<()> {
        if self.is_running() {
            return Err(WalleError::AlreadyRunning);
        }
        info!("OneBot is starting...");

        #[cfg(feature = "websocket")]
        self.ws().await?;

        #[cfg(feature = "websocket")]
        self.wsr().await?;

        self.status.store(RUNNING, Ordering::SeqCst);
        Ok(())
    }

    pub fn is_running(&self) -> bool {
        if self.status.load(Ordering::SeqCst) == SHUTDOWN {
            false
        } else {
            true
        }
    }

    pub fn is_shutdown(&self) -> bool {
        !self.is_running()
    }

    /// 关闭实例
    pub async fn shutdown(&self) {
        self.status.swap(SHUTDOWN, Ordering::SeqCst);
    }
}
