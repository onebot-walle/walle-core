use crate::{comms, Action, ActionResp, ActionRespContent, Config, Event};
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::{sync::RwLock, task::JoinHandle};
use tracing::{info, trace};

pub(crate) type CustomActionRespSender<R> = tokio::sync::oneshot::Sender<ActionResp<R>>;
// pub(crate) type ActionRespSender = CustomActionRespSender<ActionResps>;
pub(crate) type CustomActionRespMpscSender<R> = tokio::sync::mpsc::Sender<ActionResp<R>>;
// pub(crate) type ActionRespMpscSender = CustomActionRespMpscSender<ActionResps>;
#[cfg(any(feature = "http", feature = "websocket"))]
pub(crate) type CustomActionSender<A, R> = tokio::sync::mpsc::Sender<(A, CustomARSS<R>)>;
#[cfg(any(feature = "http", feature = "websocket"))]
// pub(crate) type ActionSender = CustomActionSender<Action, ActionResps>;
pub(crate) type CustomEventBroadcaster<E> = tokio::sync::broadcast::Sender<E>;
// pub(crate) type EventBroadcaster = CustomEventBroadcaster<Events>;
#[cfg(any(feature = "http", feature = "websocket"))]
pub(crate) type CustomEventListner<E> = tokio::sync::broadcast::Receiver<E>;
#[cfg(any(feature = "http", feature = "websocket"))]
// pub(crate) type EventListner = CustomEventListner<Events>;

type ArcActionHandler<A, R> = Arc<dyn crate::handle::ActionHandler<A, R> + Send + Sync>;

#[derive(Debug)]
pub enum CustomARSS<R> {
    OneShot(CustomActionRespSender<R>),
    Mpsc(CustomActionRespMpscSender<R>),
    None,
}

// pub type ARSS = CustomARSS<ActionResps>;

pub type OneBot = CustomOneBot<Event, Action, ActionRespContent>;

/// OneBot Implementation 实例
///
/// E: EventContent 可以参考 crate::evnt::EventContent
/// A: Action 可以参考 crate::action::Action
/// R: ActionRespContent 可以参考 crate::ActionRespContent
/// 如果希望包含 OneBot 的标准内容，可以使用 untagged enum 包裹。
#[allow(unused)]
pub struct CustomOneBot<E, A, R> {
    pub r#impl: String,
    pub platform: String,
    pub self_id: String,
    pub config: Config,
    action_handler: ArcActionHandler<A, R>,
    pub broadcaster: CustomEventBroadcaster<E>,
    #[cfg(feature = "http")]
    http_join_handles: RwLock<(Vec<JoinHandle<()>>, Vec<JoinHandle<()>>)>,
    #[cfg(feature = "websocket")]
    ws_join_handles: RwLock<(Vec<comms::WebSocketServer>, Vec<JoinHandle<()>>)>,
    // statu
    running: AtomicBool,
    // signal
    _shutdown: AtomicBool,
}

impl<E, A, R> CustomOneBot<E, A, R>
where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    pub fn new(
        r#impl: String,
        platform: String,
        self_id: String,
        config: Config,
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
            http_join_handles: RwLock::default(),
            ws_join_handles: RwLock::default(),
            running: AtomicBool::default(),
            _shutdown: AtomicBool::default(),
        }
    }

    /// 运行实例，该方法会永久堵塞运行，请 spawn 使用
    #[cfg(any(feature = "http", feature = "websocket"))]
    pub async fn run(&self) -> Result<(), &'static str> {
        use colored::*;

        if self.running.load(std::sync::atomic::Ordering::SeqCst) {
            return Err("OneBot is already running");
        }

        info!("{} is booting", "AbraOnebot".red());
        let (action_sender, mut action_receiver) = tokio::sync::mpsc::channel(1024);

        #[cfg(feature = "http")]
        if !self.config.http.is_empty() {
            info!("Running HTTP");
            let http_joins = &mut self.http_join_handles.write().await.0;
            for http in &self.config.http {
                http_joins.push(crate::comms::http_run(http, action_sender.clone()));
            }
        }

        #[cfg(feature = "http")]
        if !self.config.http_webhook.is_empty() {
            info!("Running HTTP Webhook");
            let webhook_joins = &mut self.http_join_handles.write().await.1;
            let clients = self.build_webhook_clients(action_sender.clone());
            for client in clients {
                webhook_joins.push(client.run());
            }
        }

        #[cfg(feature = "websocket")]
        if !self.config.websocket.is_empty() {
            info!("Running WebSocket");
            let ws_joins = &mut self.ws_join_handles.write().await.0;
            for websocket in &self.config.websocket {
                ws_joins.push(
                    crate::comms::websocket_run(
                        websocket,
                        self.broadcaster.clone(),
                        action_sender.clone(),
                    )
                    .await,
                );
            }
        }

        #[cfg(feature = "websocket")]
        if !self.config.websocket_rev.is_empty() {
            info!("Running WebSocket Reverse");
            let wsrev_joins = &mut self.ws_join_handles.write().await.1;
            for websocket_rev in &self.config.websocket_rev {
                wsrev_joins.push(
                    crate::comms::websocket_rev_run(
                        websocket_rev,
                        self.broadcaster.clone(),
                        action_sender.clone(),
                    )
                    .await,
                );
            }
        }

        trace!("Loopping to handle action and event forever");
        while let (Some((action, sender)), false) = (
            action_receiver.recv().await,
            self._shutdown.load(std::sync::atomic::Ordering::SeqCst),
        ) {
            let action_handler = self.action_handler.clone();
            tokio::spawn(async move {
                let resp = action_handler.handle(action).await;
                match sender {
                    CustomARSS::OneShot(s) => s.send(resp).unwrap(),
                    CustomARSS::Mpsc(s) => s.send(resp).await.unwrap(),
                    CustomARSS::None => {}
                }
            });
        }
        Ok(())
    }

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
        self._shutdown
            .swap(true, std::sync::atomic::Ordering::SeqCst);
        self.running
            .swap(false, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn send_event(&self, event: E) {
        match self.broadcaster.send(event) {
            Ok(t) => trace!("{} receiver receive event", t),
            Err(_) => info!("there is no event receiver yet"),
        }
    }

    pub fn new_events(&self, id: String, self_id: String, content: crate::EventContent) -> Event {
        crate::event::CustomEvent {
            id,
            r#impl: self.r#impl.clone(),
            platform: self.platform.clone(),
            self_id,
            time: crate::utils::timestamp(),
            content,
        }
    }
}
