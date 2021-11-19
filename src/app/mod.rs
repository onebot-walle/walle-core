use dashmap::DashMap;
use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::{
    config::AppConfig,
    event::BaseEvent,
    utils::{Echo, EchoS},
    Action, ActionResp, ActionRespContent, EventContent, RUNNING, SHUTDOWN,
};

mod action;

pub(crate) type ActionRespSender<R> = tokio::sync::oneshot::Sender<ActionResp<R>>;
pub(crate) type ArcEventHandler<E> =
    Arc<dyn crate::handle::EventHandler<BaseEvent<E>> + Send + Sync>;
pub(crate) type CustomActionSender<A> = tokio::sync::mpsc::UnboundedSender<Echo<A>>;
pub(crate) type CustomActionReceiver<A> = tokio::sync::mpsc::UnboundedReceiver<Echo<A>>;

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
    self_id: RwLock<String>,
    pub config: AppConfig,
    #[allow(dead_code)]
    pub(crate) event_handler: ArcEventHandler<E>,
    action_sender: CustomActionSender<A>,
    #[allow(dead_code)]
    pub(crate) action_receiver: RwLock<CustomActionReceiver<A>>,
    pub(crate) echo_map: DashMap<EchoS, ActionRespSender<R>>,

    #[cfg(feature = "websocket")]
    ws_join_handles: RwLock<(
        Option<tokio::task::JoinHandle<()>>,
        Option<crate::comms::WebSocketServer>,
    )>,

    status: AtomicU8,
}

impl<E, A, R> CustomOneBot<E, A, R>
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    pub fn new(config: AppConfig, event_handler: ArcEventHandler<E>) -> Self {
        let (action_sender, action_receiver) = tokio::sync::mpsc::unbounded_channel();
        Self {
            self_id: RwLock::default(),
            config,
            event_handler,
            action_sender,
            action_receiver: RwLock::new(action_receiver),
            echo_map: DashMap::new(),
            #[cfg(feature = "websocket")]
            ws_join_handles: RwLock::default(),
            status: AtomicU8::default(),
        }
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub async fn self_id(&self) -> String {
        self.self_id.read().await.clone()
    }

    #[allow(dead_code)]
    pub(crate) async fn set_id(&self, id: &str) {
        if &self.self_id().await != id {
            *self.self_id.write().await = id.to_owned()
        }
    }

    /// 运行 OneBot 实例
    ///
    /// 请注意该方法仅新建协程运行网络通讯协议，本身并不阻塞
    ///
    /// 当重复运行同一个实例或未设置任何通讯协议，将会返回 Err
    ///
    /// 请确保在弃用 bot 前调用 shutdown，否则无法 drop。
    pub async fn run(self: &Arc<Self>) -> Result<(), &'static str> {
        if self.status.load(Ordering::SeqCst) == RUNNING {
            return Err("OneBot is already running");
        }
        info!("OneBot is starting...");

        #[cfg(feature = "websocket")]
        if let Some(websocket) = &self.config.websocket {
            info!(target: "Walle-core", "Running WebSocket");
            self.ws_join_handles.write().await.0 =
                Some(crate::comms::app::websocket_run(websocket, self.clone()).await);
            self.status.swap(RUNNING, Ordering::SeqCst);
            return Ok(());
        }

        #[cfg(feature = "websocket")]
        if let Some(websocket_rev) = &self.config.websocket_rev {
            info!(target: "Walle-core", "Running WebSocket");
            self.ws_join_handles.write().await.1 =
                Some(crate::comms::app::websocket_rev_run(websocket_rev, self.clone()).await);
            self.status.swap(RUNNING, Ordering::SeqCst);
            return Ok(());
        }

        Err("there is no connect config found")
    }

    pub fn is_shutdown(&self) -> bool {
        if self.status.load(Ordering::SeqCst) == SHUTDOWN {
            true
        } else {
            false
        }
    }

    pub fn is_running(&self) -> bool {
        if self.status.load(Ordering::SeqCst) == SHUTDOWN {
            false
        } else {
            true
        }
    }

    /// 关闭实例
    pub async fn shutdown(&self) {
        #[cfg(feature = "websocket")]
        {
            use std::mem::swap;
            let mut joins = self.ws_join_handles.write().await;
            if let Some(j) = &joins.0 {
                j.abort();
                joins.0 = None;
            }
            if joins.1.is_some() {
                let mut j = None;
                swap(&mut joins.1, &mut j);
                j.unwrap().abort().await;
            }
        }
        self.status.swap(SHUTDOWN, Ordering::SeqCst);
    }

    pub async fn call_action(&self, action: A) {
        self.call_action_resp(action).await;
    }

    pub async fn call_action_resp(&self, action: A) -> Option<ActionResp<R>> {
        use colored::*;
        debug!(target:"Walle-core", "[{}] Sending action:{:?}", self.self_id().await.red(), action);
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let echo = EchoS::new(&self.self_id().await);
        self.echo_map.insert(echo.clone(), sender);
        self.action_sender.send(echo.clone().pack(action)).unwrap();
        match tokio::time::timeout(tokio::time::Duration::from_secs(15), async {
            if let Ok(r) = receiver.await {
                Some(r)
            } else {
                None
            }
        })
        .await
        {
            Ok(r) => r,
            Err(_) => {
                self.echo_map.remove(&echo);
                None
            }
        }
    }
}
