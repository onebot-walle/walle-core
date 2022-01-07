#![doc = include_str!("README.md")]

use crate::{event::BaseEvent, resp::StatusContent, Action, ImplConfig, WalleError, WalleResult};
use crate::{Event, HeartbeatBuild, Resps};
use serde::{de::DeserializeOwned, Serialize};
use std::fmt::Debug;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
#[cfg(any(feature = "http", feature = "websocket"))]
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{info, trace};

pub(crate) type CustomEventBroadcaster<E> = tokio::sync::broadcast::Sender<E>;
pub(crate) type ArcActionHandler<A, R> = Arc<dyn crate::handle::ActionHandler<A, R> + Send + Sync>;

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
    pub self_id: String,
    pub config: ImplConfig,
    pub(crate) action_handler: ArcActionHandler<A, R>,
    pub broadcaster: CustomEventBroadcaster<E>,
    pub(crate) ws_hooks: crate::hooks::ArcWsHooks<Self>,

    #[cfg(feature = "http")]
    http_join_handles: RwLock<(Vec<JoinHandle<()>>, Vec<JoinHandle<()>>)>,

    running: AtomicBool,
    online: AtomicBool,
}

impl<E, A, R, const V: u8> CustomOneBot<E, A, R, V> {
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

    pub(crate) fn set_running(&self) {
        self.running.swap(true, Ordering::SeqCst);
    }

    /// 关闭实例
    pub async fn shutdown(&self) {
        self.running.swap(false, Ordering::SeqCst);
    }
}

impl<E, A, R, const V: u8> CustomOneBot<E, A, R, V>
where
    E: HeartbeatBuild + Serialize + Clone + Debug + Send + 'static,
    A: DeserializeOwned + Debug + Send + 'static,
    R: Serialize + Debug + Send + 'static,
{
    pub fn new(
        r#impl: String,
        platform: String,
        self_id: String,
        config: ImplConfig,
        action_handler: ArcActionHandler<A, R>,
    ) -> Self {
        let (broadcaster, _) = tokio::sync::broadcast::channel(1024);
        Self {
            r#impl,
            platform,
            self_id,
            config,
            action_handler,
            broadcaster,
            ws_hooks: crate::hooks::empty_ws_hooks(),
            #[cfg(feature = "http")]
            http_join_handles: RwLock::default(),
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
        if !self.config.http.is_empty() {
            info!(target: "Walle-core", "Strating HTTP");
            let http_joins = &mut self.http_join_handles.write().await.0;
            for http in &self.config.http {
                http_joins.push(crate::comms::impls::http_run(
                    http,
                    self.action_handler.clone(),
                ));
            }
        }

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
        if interval <= 0 {
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
                let hb = E::build_heartbeat(&ob, interval);
                match ob.send_event(hb) {
                    _ => {}
                };
            }
        });
    }
}

impl<E, A, R, const V: u8> CustomOneBot<BaseEvent<E>, A, R, V> {
    pub fn new_event(&self, content: E) -> BaseEvent<E> {
        crate::event::BaseEvent {
            id: crate::utils::new_uuid(),
            r#impl: self.r#impl.clone(),
            platform: self.platform.clone(),
            self_id: self.self_id.clone(),
            time: crate::utils::timestamp(),
            content,
        }
    }
}

// impl<E, A, R> CustomOneBot<E, A, R>
// where
//     E: FromStandard<Event> + Serialize + Clone + Debug + Send + 'static,
// {
// pub fn new_message_event(
//     &self,
//     user_id: String,
//     group_id: Option<String>,
//     message: Message,
// ) -> E {
//     let message_c = crate::event::MessageContent {
//         ty: if let Some(group_id) = group_id {
//             crate::event::MessageEventType::Group { group_id }
//         } else {
//             crate::event::MessageEventType::Private
//         },
//         message_id: crate::utils::new_uuid(),
//         alt_message: message.alt(),
//         message,
//         user_id,
//         sub_type: "".to_owned(),
//         extra: ExtendedMap::default(),
//     };
//     self.new_event(E::from_standard(crate::event::EventContent::Message(
//         message_c,
//     )))
// }
// }
