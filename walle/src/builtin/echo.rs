use crate::{Handler, Plugin, Session};

use async_trait::async_trait;
use walle_core::MessageContent;

pub struct Echo;

#[async_trait]
impl Handler<MessageContent> for Echo {
    fn _match(&self, session: &Session<MessageContent>) -> bool {
        if session.event.content.alt_message.starts_with("echo") {
            return true;
        }
        false
    }
    async fn handle(&self, session: Session<MessageContent>) {
        let _ = session.send(session.event.message().clone()).await;
    }
}

impl Echo {
    pub fn new() -> Plugin<MessageContent> {
        Plugin::new("echo", "echo,", Echo)
    }
}

pub struct Echo2;

#[async_trait]
impl Handler<MessageContent> for Echo2 {
    fn _match(&self, session: &Session<MessageContent>) -> bool {
        if session.event.content.alt_message.starts_with("echo2") {
            return true;
        }
        false
    }
    async fn handle(&self, mut session: Session<MessageContent>) {
        let _ = session
            .get("input message", std::time::Duration::from_secs(10))
            .await;
        let _ = session.send(session.event.message().clone()).await;
    }
}

impl Echo2 {
    pub fn new() -> Plugin<MessageContent> {
        Plugin::new("echo2", "echo2,", Echo2)
    }
}
