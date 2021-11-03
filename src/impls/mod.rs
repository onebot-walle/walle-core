#![doc = include_str!("README.md")]

use crate::{
    action_resp::StatusContent, comms, event::BaseEvent, message::MessageAlt, Action, ActionResp,
    ActionRespContent, EventContent, ImplConfig, Message, RUNNING, SHUTDOWN,
};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{
    atomic::{AtomicBool, AtomicU8, Ordering},
    Arc,
};
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{info, trace};

#[cfg(any(feature = "http", feature = "websocket"))]
pub(crate) type CustomEventBroadcaster<E> = tokio::sync::broadcast::Sender<BaseEvent<E>>;
#[cfg(any(feature = "http", feature = "websocket"))]
pub(crate) type CustomEventListner<E> = tokio::sync::broadcast::Receiver<BaseEvent<E>>;
pub(crate) type ArcActionHandler<A, R> =
    Arc<dyn crate::handle::ActionHandler<A, ActionResp<R>> + Send + Sync>;

/// OneBot v12 无扩展实现端实例
pub type OneBot = CustomOneBot<EventContent, Action, ActionRespContent>;

/// OneBot Implementation 实例
///
/// E: EventContent 可以参考 crate::evnt::EventContent
/// A: Action 可以参考 crate::action::Action
/// R: ActionRespContent 可以参考 crate::action_resp::ActionRespContent
///
/// 如果希望包含 OneBot 的标准内容，可以使用 untagged enum 包裹。
pub struct CustomOneBot<E, A, R> {
    pub r#impl: String,
    pub platform: String,
    pub self_id: String,
    pub config: ImplConfig,
    action_handler: ArcActionHandler<A, R>,
    pub broadcaster: CustomEventBroadcaster<E>,

    #[cfg(feature = "http")]
    http_join_handles: RwLock<(Vec<JoinHandle<()>>, Vec<JoinHandle<()>>)>,
    #[cfg(feature = "websocket")]
    ws_join_handles: RwLock<(Vec<comms::WebSocketServer>, Vec<JoinHandle<()>>)>,

    status: AtomicU8,
    online: AtomicBool,
}

impl<E, A, R> CustomOneBot<E, A, R>
where
    E: crate::EventContentExt + Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
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
            #[cfg(feature = "http")]
            http_join_handles: RwLock::default(),
            #[cfg(feature = "websocket")]
            ws_join_handles: RwLock::default(),
            status: AtomicU8::default(),
            online: AtomicBool::default(),
        }
    }

    pub fn arc(self) -> Arc<Self> {
        Arc::new(self)
    }

    pub fn get_status(&self) -> StatusContent {
        StatusContent {
            good: if self.status.load(Ordering::SeqCst) == RUNNING {
                true
            } else {
                false
            },
            online: self.online.load(Ordering::SeqCst),
        }
    }

    /// 运行 OneBot 实例
    ///
    /// 请注意该方法仅新建协程运行网络通讯协议，本身并不阻塞
    ///
    /// 当重复运行同一个实例，将会返回 Err
    ///
    /// 请确保在弃用 bot 前调用 shutdown，否则无法 drop。
    pub async fn run(ob: Arc<Self>) -> Result<(), &'static str> {
        use colored::*;

        if ob.status.load(std::sync::atomic::Ordering::SeqCst) == RUNNING {
            return Err("OneBot is already running");
        }

        info!(target: "Walle-core", "{} is booting", ob.r#impl.red());

        #[cfg(feature = "http")]
        if !ob.config.http.is_empty() {
            info!(target: "Walle-core", "Strating HTTP");
            let http_joins = &mut ob.http_join_handles.write().await.0;
            for http in &ob.config.http {
                http_joins.push(crate::comms::impls::http_run(
                    http,
                    ob.action_handler.clone(),
                ));
            }
        }

        #[cfg(feature = "http")]
        if !ob.config.http_webhook.is_empty() {
            info!(target: "Walle-core", "Strating HTTP Webhook");
            let webhook_joins = &mut ob.http_join_handles.write().await.1;
            let clients = ob.build_webhook_clients(ob.action_handler.clone());
            for client in clients {
                webhook_joins.push(client.run());
            }
        }

        #[cfg(feature = "websocket")]
        if !ob.config.websocket.is_empty() {
            info!(target: "Walle-core", "Strating WebSocket");
            let ws_joins = &mut ob.ws_join_handles.write().await.0;
            for websocket in &ob.config.websocket {
                ws_joins.push(
                    crate::comms::impls::websocket_run(
                        websocket,
                        ob.broadcaster.clone(),
                        ob.action_handler.clone(),
                    )
                    .await,
                );
            }
        }

        #[cfg(feature = "websocket")]
        if !ob.config.websocket_rev.is_empty() {
            info!(target: "Walle-core", "Strating WebSocket Reverse");
            let wsrev_joins = &mut ob.ws_join_handles.write().await.1;
            for websocket_rev in &ob.config.websocket_rev {
                wsrev_joins.push(
                    crate::comms::impls::websocket_rev_run(
                        websocket_rev,
                        ob.broadcaster.clone(),
                        ob.action_handler.clone(),
                    )
                    .await,
                );
            }
        }
        if ob.config.heartbeat.enabled {
            ob.start_heartbeat(ob.clone());
        }

        ob.status.swap(RUNNING, std::sync::atomic::Ordering::SeqCst);
        Ok(())
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
        #[cfg(feature = "http")]
        {
            let mut joins = self.http_join_handles.write().await;
            while !joins.0.is_empty() {
                joins.0.pop().unwrap().abort()
            }
            while !joins.1.is_empty() {
                joins.1.pop().unwrap().abort()
            }
        }
        #[cfg(feature = "websocket")]
        {
            let mut joins = self.ws_join_handles.write().await;
            while !joins.0.is_empty() {
                joins.0.pop().unwrap().abort().await
            }
            while !joins.1.is_empty() {
                joins.1.pop().unwrap().abort()
            }
        }
        self.status
            .swap(SHUTDOWN, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn send_event(&self, event: BaseEvent<E>) -> Result<usize, &str> {
        match self.broadcaster.send(event) {
            Ok(t) => Ok(t),
            Err(_) => Err("there is no event receiver can receive the event yet"),
        }
    }

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

    pub fn new_message_event(
        &self,
        user_id: String,
        group_id: Option<String>,
        message: Message,
    ) -> BaseEvent<E> {
        let message_c = crate::event::Message {
            ty: if let Some(group_id) = group_id {
                crate::event::MessageEventType::Group { group_id }
            } else {
                crate::event::MessageEventType::Private
            },
            message_id: crate::utils::new_uuid(),
            alt_message: message.alt(),
            message,
            user_id,
            sub_type: "".to_owned(),
        };
        self.new_event(E::from_standard(crate::event::EventContent::Message(
            message_c,
        )))
    }

    fn start_heartbeat(&self, ob: Arc<Self>) {
        let mut interval = self.config.heartbeat.interval;
        if interval <= 0 {
            interval = 4;
        }
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(interval as u64)).await;
                if ob.status.load(Ordering::SeqCst) != RUNNING {
                    break;
                }
                trace!(target:"Walle-core", "Heartbeating");
                let hb = ob.new_event(E::from_standard(EventContent::Meta(
                    crate::event::Meta::Heartbeat {
                        interval,
                        status: ob.get_status(),
                        sub_type: "".to_owned(),
                    },
                )));
                match ob.send_event(hb) {
                    _ => {}
                };
            }
        });
    }
}
