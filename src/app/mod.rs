use std::{
    collections::HashMap,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    config::AppConfig, StandardAction, Event, ProtocolItem, Resps, SelfId, WalleError, WalleResult,
};

mod bot;

pub(crate) type ArcEventHandler<E, A, R> =
    Arc<dyn crate::handle::EventHandler<E, A, R> + Send + Sync>;
pub(crate) type CustomRespSender<R> = tokio::sync::oneshot::Sender<R>;
pub(crate) type CustomActionSender<A, R> =
    tokio::sync::mpsc::UnboundedSender<(A, CustomRespSender<R>)>;

/// OneBot v12 无扩展应用端实例
pub type OneBot = CustomOneBot<Event, StandardAction, Resps, 12>;

/// OneBot Application 实例
///
/// E: Event 可以参考 crate::evnt::Event
/// A: Action 可以参考 crate::action::Action
/// R: ActionResp 可以参考 crate::action_resp::ActionResps
/// V: OneBot 协议版本号
///
/// 如果希望包含 OneBot 的标准内容，可以使用 untagged enum 包裹。
pub struct CustomOneBot<E, A, R, const V: u8> {
    pub config: AppConfig,
    pub bots: RwLock<HashMap<String, ArcBot<A, R>>>,

    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub(crate) ws_hooks: crate::hooks::ArcWsHooks<Self>,
    pub(crate) event_handler: ArcEventHandler<E, A, R>,

    running: AtomicBool,
}

/// Arc<Bot>
pub type ArcBot<A, R> = Arc<Bot<A, R>>;

/// Bot 实例
pub struct Bot<A, R> {
    #[allow(dead_code)]
    self_id: String,
    sender: CustomActionSender<A, R>,
}

impl<E, A, R, const V: u8> CustomOneBot<E, A, R, V>
where
    E: ProtocolItem + SelfId + Clone + Send + 'static + Debug,
    A: ProtocolItem + Clone + Send + 'static + Debug,
    R: ProtocolItem + Clone + Send + 'static + Debug,
{
    /// 创建新的 OneBot 实例
    pub fn new(config: AppConfig, event_handler: ArcEventHandler<E, A, R>) -> Self {
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
    pub async fn run(self: &Arc<Self>) -> WalleResult<()> {
        if self.is_running() {
            return Err(WalleError::AlreadyRunning);
        }
        info!(target: "Walle-core", "OneBot is starting...");

        #[cfg(feature = "websocket")]
        self.ws().await;

        #[cfg(feature = "websocket")]
        self.wsr().await?;

        self.running.store(true, Ordering::Relaxed);
        Ok(())
    }

    pub async fn run_block(self: &Arc<Self>) -> WalleResult<()> {
        if self.is_running() {
            return Err(WalleError::AlreadyRunning);
        }
        info!(target: "Walle-core", "OneBot is starting...");

        #[cfg(feature = "websocket")]
        {
            let mut joins = self.ws().await;
            for join in self.wsr().await? {
                joins.push(join);
            }
            if !joins.is_empty() {
                self.running.store(true, Ordering::Relaxed);
            }
            for join in joins {
                let _ = join.await;
            }
        }
        Ok(())
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
}
