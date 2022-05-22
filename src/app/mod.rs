use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tracing::info;

use crate::{
    config::AppConfig, handle::EventHandler, ProtocolItem, Resps, SelfId, StandardAction,
    StandardEvent, WalleError, WalleResult,
};

mod bot;

pub(crate) use tokio::sync::oneshot::Sender as OneshotSender;
pub(crate) type CustomActionSender<A, R> =
    tokio::sync::mpsc::UnboundedSender<(A, Option<OneshotSender<R>>)>;

/// OneBot v12 无扩展应用端实例
pub type StandardOneBot<H> = OneBot<StandardEvent, StandardAction, Resps<StandardEvent>, H, 12>;

/// OneBot Application 实例
///
/// E: Event 可以参考 crate::evnt::Event
/// A: Action 可以参考 crate::action::Action
/// R: ActionResp 可以参考 crate::action_resp::ActionResps
/// H: EventHandler 需要实现 trait `EventHandler<E, A, R>`
/// V: OneBot 协议版本号
///
/// 如果希望包含 OneBot 的标准内容，可以使用 untagged enum 包裹。
pub struct OneBot<E, A, R, H, const V: u8> {
    pub config: AppConfig,
    pub bots: RwLock<HashMap<String, ArcBot<A, R>>>,

    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub(crate) ws_hooks: crate::hooks::BoxWsHooks<Self>,
    pub(crate) event_handler: H,

    running: AtomicBool,
}

/// Arc<Bot>
pub type ArcBot<A, R> = Arc<Bot<A, R>>;
pub type StandardArcBot = ArcBot<StandardAction, Resps<StandardEvent>>;

/// Bot 实例
pub struct Bot<A, R> {
    #[allow(dead_code)]
    pub self_id: String,
    sender: CustomActionSender<A, R>,
}

impl<E, A, R, H, const V: u8> OneBot<E, A, R, H, V>
where
    E: Sync + Send + 'static,
    A: Sync + Send + 'static,
    R: Sync + Send + 'static,
    H: Sync + Send + 'static,
{
    /// 创建新的 OneBot 实例
    pub fn new(config: AppConfig, event_handler: H) -> Self {
        Self {
            config,
            event_handler,
            running: AtomicBool::default(),
            bots: RwLock::default(),
            #[cfg(feature = "websocket")]
            #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
            ws_hooks: crate::hooks::empty_ws_hooks(),
        }
    }
}

impl<E, A, R, H, const V: u8> OneBot<E, A, R, H, V> {
    /// 返回 Arc<OneBot>
    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    /// 根据 bot_id 获取 Bot 实例
    pub async fn get_bot(&self, bot_id: &str) -> Option<ArcBot<A, R>> {
        self.bots.read().await.get(bot_id).cloned()
    }

    /// 获取所有 Bot 实例
    pub async fn get_bots(&self) -> HashMap<String, ArcBot<A, R>> {
        self.bots.read().await.clone()
    }

    /// 返回 OneBot 实例是否运行中
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// 返回 OneBot 实例是否停止运行
    pub fn is_shutdown(&self) -> bool {
        !self.is_running()
    }

    /// 关闭 OneBot 实例
    pub async fn shutdown(&self) {
        self.running.swap(false, Ordering::SeqCst);
    }

    pub(crate) fn set_running(&self) {
        self.running.swap(true, Ordering::SeqCst);
    }
}

impl<E, A, R, H, const V: u8> OneBot<E, A, R, H, V>
where
    E: ProtocolItem + SelfId + Clone + Send + 'static + Debug,
    A: ProtocolItem + Clone + Send + 'static + Debug,
    R: ProtocolItem + Clone + Send + 'static + Debug,
    H: EventHandler<E, A, R> + Send + Sync + 'static,
{
    /// 添加 Bot 实例
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

    /// 删除 Bot 实例
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
    pub async fn run(self: &Arc<Self>) -> WalleResult<Vec<JoinHandle<()>>> {
        if self.is_running() {
            return Err(WalleError::AlreadyRunning);
        }
        info!(target: crate::WALLE_CORE, "OneBot is starting...");
        let mut joins = vec![];

        #[cfg(feature = "http")]
        self.http(&mut joins).await;

        #[cfg(feature = "websocket")]
        self.ws(&mut joins).await;

        #[cfg(feature = "websocket")]
        self.wsr(&mut joins).await?;

        Ok(joins)
    }

    pub async fn run_block(self: &Arc<Self>) -> WalleResult<()> {
        for join in self.run().await? {
            let _ = join.await;
        }
        Ok(())
    }
}
