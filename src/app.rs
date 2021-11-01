use std::sync::{
    atomic::{AtomicU8, Ordering},
    Arc,
};
use tokio::{sync::RwLock, task::JoinHandle};

use crate::{
    config::AppConfig, event::BaseEvent, Action, ActionResps, EventContent, RUNNING, SHUTDOWN,
};

pub(crate) type ActionRespSender<R> = tokio::sync::mpsc::Sender<R>;
pub(crate) type ArcEventHandler<E> =
    Arc<dyn crate::handle::EventHandler<BaseEvent<E>> + Send + Sync>;
pub(crate) type CustomActionBroadcaster<A, R> =
    tokio::sync::broadcast::Sender<(A, ActionRespSender<R>)>;
pub(crate) type CustomActionListenr<A, R> =
    tokio::sync::broadcast::Receiver<(A, ActionRespSender<R>)>;

/// OneBot v12 无扩展应用端实例
pub type OneBot = CustomOneBot<EventContent, Action, ActionResps>;

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
    event_handler: ArcEventHandler<E>,
    action_broadcaster: CustomActionBroadcaster<A, R>,

    #[cfg(feature = "websocket")]
    ws_join_handles: RwLock<(
        Option<JoinHandle<()>>,
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
        let (action_broadcaster, _) = tokio::sync::broadcast::channel(1024);
        Self {
            config,
            event_handler,
            action_broadcaster,
            #[cfg(feature = "websocket")]
            ws_join_handles: RwLock::default(),
            status: AtomicU8::default(),
        }
    }

    /// 运行 OneBot 实例
    ///
    /// 请注意该方法仅新建协程运行网络通讯协议，本身并不阻塞
    ///
    /// 当重复运行同一个实例或未设置任何通讯协议，将会返回 Err
    pub async fn run(&self) -> Result<(), &str> {
        use tracing::info;

        if self.status.load(std::sync::atomic::Ordering::SeqCst) == RUNNING {
            return Err("OneBot is already running");
        }

        #[cfg(feature = "websocket")]
        if let Some(websocket) = &self.config.websocket {
            info!(target: "Walle-core", "Running WebSocket");
            self.ws_join_handles.write().await.0 = Some(
                crate::comms::app::websocket_run(
                    websocket,
                    self.event_handler.clone(),
                    self.action_broadcaster.clone(),
                )
                .await,
            );
            self.status.swap(RUNNING, Ordering::SeqCst);
            return Ok(());
        }

        #[cfg(feature = "websocket")]
        if let Some(websocket_rev) = &self.config.websocket_rev {
            info!(target: "Walle-core", "Running WebSocket");
            self.ws_join_handles.write().await.1 = Some(
                crate::comms::app::websocket_rev_run(
                    websocket_rev,
                    self.event_handler.clone(),
                    self.action_broadcaster.clone(),
                )
                .await,
            );
            self.status.swap(RUNNING, Ordering::SeqCst);
            return Ok(());
        }

        Err("there is no connect config found")
    }

    pub fn is_shutdown(&self) -> bool {
        if self.status.load(std::sync::atomic::Ordering::SeqCst) == SHUTDOWN {
            true
        } else {
            false
        }
    }

    pub fn is_running(&self) -> bool {
        if self.status.load(std::sync::atomic::Ordering::SeqCst) == SHUTDOWN {
            false
        } else {
            true
        }
    }

    /// 关闭实例
    pub async fn shutdown(&self) {
        use std::mem::swap;
        #[cfg(feature = "websocket")]
        {
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
        self.status
            .swap(SHUTDOWN, std::sync::atomic::Ordering::SeqCst);
    }

    pub async fn call_action(&self, action: A) {
        let (sender, _) = tokio::sync::mpsc::channel(1);
        self.action_broadcaster.send((action, sender)).unwrap();
    }

    pub async fn call_action_resp(&self, action: A) -> Option<R> {
        let (sender, mut receiver) = tokio::sync::mpsc::channel(1);
        self.action_broadcaster.send((action, sender)).unwrap();
        match tokio::time::timeout(tokio::time::Duration::from_secs(15), async {
            if let Some(r) = receiver.recv().await {
                Some(r)
            } else {
                None
            }
        })
        .await
        {
            Ok(r) => r,
            Err(_) => None,
        }
    }
}
