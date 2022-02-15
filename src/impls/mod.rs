#![doc = include_str!("README.md")]

use crate::{event::BaseEvent, resp::StatusContent, Action, ImplConfig, WalleError, WalleResult};
use crate::{Event, HeartbeatBuild, Resps};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, trace};

pub type CustomEventBroadcaster<E> = tokio::sync::broadcast::Sender<E>;
pub(crate) type ArcActionHandler<A, R, OB> =
    Arc<dyn crate::handle::ActionHandler<A, R, OB> + Send + Sync>;
pub type EventBroadcaster = CustomEventBroadcaster<Event>;

/// OneBot v12 无扩展实现端实例
pub type OneBot = CustomOneBot<Event, Action, Resps, 12>;

/// OneBot Implementation 实例
///
/// E: Event 可以参考 crate::evnt::Event
/// A: Action 可以参考 crate::action::Action
/// R: ActionResp 可以参考 crate::action_resp::Resps
/// V: OneBot 协议版本号
///
/// 如果希望包含 OneBot 的标准内容，可以使用 untagged enum 包裹。
pub struct CustomOneBot<E, A, R, const V: u8> {
    pub r#impl: String,
    pub platform: String,
    pub self_id: RwLock<String>,
    pub config: ImplConfig,
    pub broadcaster: CustomEventBroadcaster<E>,

    pub action_handler: ArcActionHandler<A, R, Self>,
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    pub(crate) ws_hooks: crate::hooks::ArcWsHooks<Self>,

    running: AtomicBool,
    online: AtomicBool,
}

impl<E, A, R, const V: u8> CustomOneBot<E, A, R, V> {
    pub async fn self_id(&self) -> String {
        self.self_id.read().await.clone()
    }

    pub fn onebot_version() -> u8 {
        V
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn get_status(&self) -> StatusContent {
        StatusContent {
            good: self.is_running(),
            online: self.online.load(Ordering::SeqCst),
        }
    }

    pub fn is_shutdown(&self) -> bool {
        !self.is_running()
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    pub fn set_online(&self, online: bool) {
        self.online.store(online, Ordering::SeqCst);
    }

    /// 关闭实例
    pub async fn shutdown(&self) {
        self.running.swap(false, Ordering::SeqCst);
    }

    pub(crate) fn set_running(&self) {
        self.running.swap(true, Ordering::SeqCst);
    }
}

impl<E, A, R, const V: u8> CustomOneBot<E, A, R, V>
where
    E: HeartbeatBuild + Serialize + Clone + Debug + Send + 'static,
    A: DeserializeOwned + Clone + Debug + Send + 'static,
    R: Serialize + Clone + Debug + Send + 'static,
{
    pub fn new(
        r#impl: &str,
        platform: &str,
        self_id: &str,
        config: ImplConfig,
        action_handler: ArcActionHandler<A, R, Self>,
    ) -> Self {
        let (broadcaster, _) = tokio::sync::broadcast::channel(1024);

        let mut rx = broadcaster.subscribe(); // avoid no receiver error
        tokio::spawn(async move {
            loop {
                if rx.recv().await.is_err() {
                    break;
                }
            }
        });

        Self {
            r#impl: r#impl.to_owned(),
            platform: platform.to_owned(),
            self_id: RwLock::new(self_id.to_owned()),
            config,
            action_handler,
            broadcaster,
            #[cfg(feature = "websocket")]
            #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
            ws_hooks: crate::hooks::empty_ws_hooks(),
            running: AtomicBool::default(),
            online: AtomicBool::default(),
        }
    }

    /// 运行 OneBot 实例
    ///
    /// 请注意该方法仅新建协程运行网络通讯协议，本身并不阻塞
    ///
    /// 当重复运行同一个实例，将会返回 Err
    ///
    /// 请确保在弃用 bot 前调用 shutdown，否则无法 drop。
    pub async fn run(self: &Arc<Self>) -> WalleResult<()> {
        use colored::*;

        if self.is_running() {
            return Err(WalleError::AlreadyRunning);
        }

        info!(target: "Walle-core", "{} is booting", self.r#impl.red());

        #[cfg(feature = "http")]
        self.http().await?;

        #[cfg(feature = "http")]
        self.webhook().await;

        #[cfg(feature = "websocket")]
        self.ws().await?;

        #[cfg(feature = "websocket")]
        self.wsr().await;

        if self.config.heartbeat.enabled {
            self.start_heartbeat();
        }

        Ok(())
    }

    pub fn send_event(&self, event: E) -> Result<usize, &str> {
        match self.broadcaster.send(event) {
            Ok(t) => Ok(t),
            Err(_) => Err("there is no event receiver can receive the event yet"),
        }
    }

    fn start_heartbeat(self: &Arc<Self>) {
        let mut interval = self.config.heartbeat.interval;
        if interval == 0 {
            interval = 4;
        }
        let ob = self.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(interval as u64)).await;
                if ob.is_shutdown() {
                    break;
                }
                trace!(target:"Walle-core", "Heartbeating");
                let hb = E::build_heartbeat(&ob, interval).await;
                let _ = ob.send_event(hb);
            }
        });
    }
}

impl<E, A, R, const V: u8> CustomOneBot<BaseEvent<E>, A, R, V> {
    pub async fn new_event(&self, content: E, time: u64) -> BaseEvent<E> {
        crate::event::BaseEvent {
            id: crate::utils::new_uuid(),
            r#impl: self.r#impl.clone(),
            platform: self.platform.clone(),
            self_id: self.self_id.read().await.clone(),
            time,
            content,
        }
    }
}
