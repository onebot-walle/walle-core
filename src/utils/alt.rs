use colored::*;

use crate::{StandardAction, BaseEvent, EventContent, MessageAlt, MessageContent};

impl<T: ColoredAlt> ColoredAlt for BaseEvent<T> {
    fn alt(&self) -> Option<String> {
        self.content.alt()
    }
}

pub trait ColoredAlt {
    fn alt(&self) -> Option<String>;
}

impl ColoredAlt for EventContent {
    fn alt(&self) -> Option<String> {
        match self {
            EventContent::Message(m) => m.alt(),
            _ => None, //todo
        }
    }
}

impl ColoredAlt for MessageContent {
    fn alt(&self) -> Option<String> {
        match &self.ty {
            crate::MessageEventType::Group { group_id } => Some(format!(
                "[{}] {} from {}",
                group_id.bright_blue(),
                self.alt_message,
                self.user_id.bright_green()
            )),
            crate::MessageEventType::Private => Some(format!(
                "[{}] {}",
                self.user_id.bright_green(),
                self.alt_message
            )),
        }
    }
}

impl ColoredAlt for StandardAction {
    fn alt(&self) -> Option<String> {
        match self {
            StandardAction::SendMessage(c) => {
                if let Some(group_id) = &c.group_id {
                    Some(format!(
                        "[{}] {} to {}",
                        "SendMessage".bright_yellow(),
                        c.message.alt(),
                        group_id.bright_blue(),
                    ))
                } else if let Some(user_id) = &c.user_id {
                    Some(format!(
                        "[{}] {} to {}",
                        "SendMessage".bright_yellow(),
                        c.message.alt(),
                        user_id.bright_green()
                    ))
                } else {
                    None
                }
            }
            _ => None, //todo
        }
    }
}
