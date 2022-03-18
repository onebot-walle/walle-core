use colored::*;
use walle_core::ColoredAlt;

impl ColoredAlt for super::Event {
    fn alt(&self) -> Option<String> {
        self.content.alt()
    }
}

impl ColoredAlt for super::EventContent {
    fn alt(&self) -> Option<String> {
        match self {
            super::EventContent::Message(m) => m.alt(),
            _ => None, //todo
        }
    }
}

impl ColoredAlt for super::MessageContent {
    fn alt(&self) -> Option<String> {
        match &self.sub {
            super::MessageSub::Group { group_id, .. } => Some(format!(
                "[{}] {} from {}",
                group_id.to_string().bright_blue(),
                self.raw_message,
                self.user_id.to_string().bright_green()
            )),
            super::MessageSub::Private { .. } => Some(format!(
                "[{}] {}",
                self.user_id.to_string().bright_green(),
                self.raw_message
            )),
        }
    }
}
