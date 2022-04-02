use crate::{Matcher, Plugin, Session};

use async_trait::async_trait;
use walle_core::{EventContent, MessageBuild, StandardEvent};

pub struct Echo;

#[async_trait]
impl Matcher for Echo {
    fn _match(&self, event: &StandardEvent) -> bool {
        if let EventContent::Message(ref c) = event.content {
            if c.alt_message.starts_with("echo") {
                return true;
            }
        }
        false
    }
    async fn handle(&self, session: Session<EventContent>) {
        let session = session.as_message_session().unwrap();
        let _ = session.send(session.event.message().clone()).await;
    }
}

impl Echo {
    pub fn new() -> Plugin {
        Plugin::new("echo", "echo,", Echo)
    }
}

pub struct Echo2;

#[async_trait]
impl Matcher for Echo2 {
    fn _match(&self, event: &StandardEvent) -> bool {
        if let EventContent::Message(ref c) = event.content {
            if c.alt_message.starts_with("echo2") {
                return true;
            }
        }
        false
    }
    async fn handle(&self, session: Session<EventContent>) {
        let mut session = session.as_message_session().unwrap();
        let _ = session
            .get(
                vec![].text("input message".to_string()),
                std::time::Duration::from_secs(10),
            )
            .await;
        let _ = session.send(session.event.message().clone()).await;
    }
}

impl Echo2 {
    pub fn new() -> Plugin {
        Plugin::new("echo2", "echo2,", Echo2)
    }
}
