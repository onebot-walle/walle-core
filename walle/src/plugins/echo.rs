use crate::{Matcher, Plugin, Session};

use async_trait::async_trait;
use walle_core::{EventContent, MessageEventType};

pub struct Echo;

#[async_trait]
impl Matcher for Echo {
    async fn handle(&self, session: Session<EventContent>) {
        if let EventContent::Message(c) = session.event.content {
            match c.ty {
                MessageEventType::Group { group_id } => {
                    session
                        .bot
                        .send_group_message(group_id, c.message.clone())
                        .await
                        .unwrap();
                }
                MessageEventType::Private => {
                    session
                        .bot
                        .send_private_message(c.user_id, c.message.clone())
                        .await
                        .unwrap();
                }
            }
        }
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
    async fn handle(&self, _session: Session<EventContent>) {}
}

impl Echo2 {
    pub fn new() -> Plugin {
        Plugin::new("echo2", "echo2,", Echo2)
    }
}
