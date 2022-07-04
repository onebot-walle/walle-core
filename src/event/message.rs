use crate::util::ExtendedMap;

use serde::{Deserialize, Serialize};
use snake_cased::SnakedEnum;

/// ## OneBot 消息事件 Content
///
/// 消息事件是聊天机器人收到其他用户发送的消息对应的一类事件，例如私聊消息等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageContent<D> {
    #[serde(flatten)]
    pub detail: D,
    pub message_id: String,
    pub message: crate::message::Message,
    pub alt_message: String,
    pub user_id: String,
}

/// MessageEvent detail_type ( private or group )
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SnakedEnum)]
#[serde(tag = "detail_type", rename_all = "snake_case")]
pub enum MessageEventDetail {
    Private {
        /// just for Deserialize
        sub_type: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    Group {
        /// just for Deserialize
        sub_type: String,
        group_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    Channel {
        /// just for Deserialize
        sub_type: String,
        guild_id: String,
        channel_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
}

impl super::EventSubType for MessageEventDetail {
    fn sub_type(&self) -> &str {
        match self {
            Self::Group { sub_type, .. } => sub_type,
            Self::Channel { sub_type, .. } => sub_type,
            Self::Private { sub_type, .. } => sub_type,
        }
    }
}

impl super::EventDetailType for MessageEventDetail {
    fn detail_type(&self) -> &str {
        self.snaked_enum()
    }
}

impl<D: super::EventSubType> super::EventSubType for MessageContent<D> {
    fn sub_type(&self) -> &str {
        self.detail.sub_type()
    }
}

impl<D: super::EventDetailType> super::EventDetailType for MessageContent<D> {
    fn detail_type(&self) -> &str {
        self.detail.detail_type()
    }
}

impl<D: super::EventDetailType> super::EventType for MessageContent<D> {
    fn ty(&self) -> &str {
        "message"
    }
}
