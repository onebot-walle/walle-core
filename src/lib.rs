mod action;
mod action_resp;
mod comms;
mod config;
mod event;
mod event_builder;
mod handle;
mod message;
mod test;

pub use action::*;
pub use action_resp::*;
pub use config::Config;
pub use event::{Events, MessageEvent, MetaEvent, NoticeEvent, RequestEvent};
pub use message::{Message, MessageBuild, MessageSegment};

pub type EventSender = tokio::sync::mpsc::Sender<Events>;
type EventReceiver = tokio::sync::mpsc::Receiver<Events>;
type ActionRespSender = tokio::sync::oneshot::Sender<Option<ActionResps>>;
type ActionRespMpscSender = tokio::sync::mpsc::Sender<Option<ActionResps>>;

#[derive(Debug)]
pub enum ARSS {
    OneShot(ActionRespSender),
    Mpsc(ActionRespMpscSender),
    None,
}

#[cfg(any(feature = "http", feature = "websocket"))]
type ActionSender = tokio::sync::mpsc::Sender<(Action, ARSS)>;
type EventBroadcaster = tokio::sync::broadcast::Sender<Events>;
#[cfg(any(feature = "http", feature = "websocket"))]
type EventListner = tokio::sync::broadcast::Receiver<Events>;

/// OneBot 实例
#[allow(unused)]
pub struct OneBot {
    r#impl: String,
    platform: String,
    self_id: String,
    config: Config,
    event_receiver: EventReceiver,
    action_handler: Box<dyn handle::ActionHandler>,
    broadcaster: EventBroadcaster,
}

impl OneBot {
    pub fn new(
        r#impl: String,
        platform: String,
        self_id: String,
        config: Config,
        event_receiver: EventReceiver,
        action_handler: Box<dyn handle::ActionHandler>,
    ) -> Self {
        let (broadcaster, _) = tokio::sync::broadcast::channel(1024);
        Self {
            r#impl,
            platform,
            self_id,
            config,
            event_receiver,
            action_handler,
            broadcaster,
        }
    }

    #[cfg(any(feature = "http", feature = "websocket"))]
    pub async fn run(mut self) {
        let (action_sender, mut action_receiver) = tokio::sync::mpsc::channel(1024);
        #[cfg(feature = "http")]
        for http in &self.config.http {
            crate::comms::http_run(http, action_sender.clone());
        }
        #[cfg(feature = "http")]
        {
            let clients = self.build_webhook_clients(action_sender.clone());
            for client in clients {
                client.run().await;
            }
        }
        #[cfg(feature = "websocket")]
        for websocket in &self.config.websocket {
            crate::comms::websocket_run(websocket, self.broadcaster.clone(), action_sender.clone())
                .await;
        }
        #[cfg(feature = "websocket")]
        for websocket_rev in &self.config.websocket_rev {
            crate::comms::websocket_rev_run(
                websocket_rev,
                self.broadcaster.clone(),
                action_sender.clone(),
            )
            .await;
        }
        loop {
            tokio::select! {
                option_action = action_receiver.recv() => {
                    if let Some((action, sender)) = option_action {
                        let resp = self.action_handler.handle(action).await;
                        match sender {
                            ARSS::OneShot(s) => s.send(resp).unwrap(),
                            ARSS::Mpsc(s) => s.send(resp).await.unwrap(),
                            ARSS::None => {}
                        }
                    }
                }
                option_event = self.event_receiver.recv() => {
                    if let Some(event) = option_event {
                        self.broadcaster.send(event).unwrap();
                    }
                }
            }
        }
    }
}
