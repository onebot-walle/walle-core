use colored::*;

use crate::event::{
    EventContent, MessageContent, MessageEventDetail, NoticeContent, RequestContent,
};
use crate::{BaseEvent, MessageAlt, StandardAction};

pub trait ColoredAlt {
    fn colored_alt(&self) -> Option<String>;
}

impl<T: ColoredAlt> ColoredAlt for BaseEvent<T> {
    fn colored_alt(&self) -> Option<String> {
        self.content.colored_alt()
    }
}

impl ColoredAlt for EventContent {
    fn colored_alt(&self) -> Option<String> {
        match self {
            EventContent::Message(m) => m.colored_alt(),
            EventContent::Notice(n) => n.colored_alt(),
            EventContent::Request(r) => r.colored_alt(),
            _ => None,
        }
    }
}

impl ColoredAlt for MessageContent<MessageEventDetail> {
    fn colored_alt(&self) -> Option<String> {
        match &self.detail {
            MessageEventDetail::Channel {
                guild_id,
                channel_id,
                ..
            } => Some(format!(
                "[{}:{}] {} from {}",
                guild_id.blue(),
                channel_id.bright_blue(),
                self.alt_message,
                self.user_id.bright_green()
            )),
            MessageEventDetail::Group { group_id, .. } => Some(format!(
                "[{}] {} from {}",
                group_id.bright_blue(),
                self.alt_message,
                self.user_id.bright_green()
            )),
            MessageEventDetail::Private { .. } => Some(format!(
                "[{}] {}",
                self.user_id.bright_green(),
                self.alt_message
            )),
        }
    }
}

impl ColoredAlt for NoticeContent {
    fn colored_alt(&self) -> Option<String> {
        let head = format!("[{}]", self.detail_type().bright_red());
        let body = match self {
            Self::GroupMemberIncrease {
                sub_type,
                group_id,
                user_id,
                operator_id,
                ..
            } => match sub_type.as_str() {
                "invite" => format!(
                    "{} invite {} to {}",
                    operator_id.bright_red(),
                    user_id.bright_green(),
                    group_id.bright_blue()
                ),
                "join" => format!("{} join {}", user_id.bright_green(), group_id.bright_blue()),
                _ => format!("{:?}", self),
            },
            Self::GroupMemberDecrease {
                sub_type,
                group_id,
                user_id,
                operator_id,
                ..
            } => match sub_type.as_str() {
                "kick" => format!(
                    "{} kick {} out of {}",
                    operator_id.bright_red(),
                    user_id.bright_green(),
                    group_id.bright_blue()
                ),
                "leave" => format!(
                    "{} leave {}",
                    user_id.bright_green(),
                    group_id.bright_blue()
                ),
                _ => format!("{:?}", self),
            },
            _ => format!("{:?}", self), //todo
        };
        return Some(format!("{} {}", head, body));
    }
}

impl ColoredAlt for RequestContent {
    fn colored_alt(&self) -> Option<String> {
        return Some(format!(
            "[{}] {:?}",
            self.detail_type().bright_yellow(),
            self
        ));
    }
}

impl ColoredAlt for StandardAction {
    fn colored_alt(&self) -> Option<String> {
        match self {
            StandardAction::SendMessage(c) => {
                if let Some(group_id) = &c.group_id {
                    Some(format!(
                        "[{}] {} to {}",
                        "SendMessage".bright_yellow(),
                        c.message.alt(),
                        group_id.bright_blue(),
                    ))
                } else {
                    c.user_id.as_ref().map(|user_id| {
                        format!(
                            "[{}] {} to {}",
                            "SendMessage".bright_yellow(),
                            c.message.alt(),
                            user_id.bright_green()
                        )
                    })
                }
            }
            _ => None, //todo
        }
    }
}
