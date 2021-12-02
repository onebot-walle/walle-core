use crate::handler::{default_handler, ArcEventHandler};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicBool, AtomicI32},
    Arc,
};
use tokio::sync::RwLock;

pub(crate) type ActionSender = tokio::sync::mpsc::UnboundedSender<crate::action::Action>;

pub struct OneBot {
    handler: ArcEventHandler,
    action_sender: RwLock<Option<ActionSender>>,
    config: crate::config::AppConfig,
    running: AtomicBool,
}

impl OneBot {
    pub fn new(handler: ArcEventHandler, config: crate::config::AppConfig) -> Self {
        Self {
            handler,
            action_sender: RwLock::default(),
            config,
            running: AtomicBool::new(false),
        }
    }

    pub async fn run(self: &Arc<Self>) {
        if let Some(wsr) = &self.config.web_socket {
            let mut rx = crate::ws::wsr(
                wsr.clone(),
                (
                    default_handler(),
                    self.handler.clone(),
                    tokio::sync::broadcast::channel(0).0,
                    tokio::sync::broadcast::channel(0).0,
                ),
            )
            .await;
            let ob = self.clone();
            tokio::spawn(async move {
                while let Some(sender) = rx.recv().await {
                    ob.action_sender.write().await.replace(sender);
                }
            });
            self.running
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }
        if !self.running.load(std::sync::atomic::Ordering::SeqCst) {
            if let Some(ws) = &self.config.web_socket_rev {
                let mut rx = crate::ws::ws(
                    ws.clone(),
                    (
                        default_handler(),
                        self.handler.clone(),
                        tokio::sync::broadcast::channel(0).0,
                        tokio::sync::broadcast::channel(0).0,
                    ),
                )
                .await;
                let ob = self.clone();
                tokio::spawn(async move {
                    while let Some(sender) = rx.recv().await {
                        ob.action_sender.write().await.replace(sender);
                    }
                });
                self.running
                    .store(true, std::sync::atomic::Ordering::SeqCst);
            }
        }
    }
}
