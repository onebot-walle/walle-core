use super::BaseEvent;
use crate::ExtendedMap;
#[cfg(feature = "impl")]
use crate::MessageAlt;
use serde::{Deserialize, Serialize};

/// ## OneBot 消息事件 Content
///
/// 消息事件是聊天机器人收到其他用户发送的消息对应的一类事件，例如私聊消息等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageContent {
    #[serde(flatten)]
    pub ty: MessageEventType,
    pub message_id: String,
    pub message: crate::Message,
    pub alt_message: String,
    pub user_id: String,
    /// just for Deserialize
    pub sub_type: String,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

/// MessageEvent detail_type ( private or group )
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type", rename_all = "snake_case")]
pub enum MessageEventType {
    Private,
    Group { group_id: String },
}

impl MessageEventType {
    pub fn group_id(&self) -> Option<&str> {
        match self {
            MessageEventType::Group { group_id } => Some(group_id),
            _ => None,
        }
    }
}

#[cfg(feature = "impl")]
impl MessageContent {
    pub fn new_group_message_content(
        message: crate::Message,
        message_id: String,
        user_id: String,
        group_id: String,
        extra: ExtendedMap,
    ) -> Self {
        Self {
            ty: MessageEventType::Group { group_id },
            message_id,
            alt_message: message.alt(),
            message,
            user_id,
            sub_type: "".to_owned(),
            extra,
        }
    }

    pub fn new_private_message_content(
        message: crate::Message,
        message_id: String,
        user_id: String,
        extra: ExtendedMap,
    ) -> Self {
        Self {
            ty: MessageEventType::Private,
            message_id,
            alt_message: message.alt(),
            message,
            user_id,
            sub_type: "".to_owned(),
            extra,
        }
    }
}

impl BaseEvent<MessageContent> {
    pub fn group_id(&self) -> Option<&str> {
        self.content.ty.group_id()
    }
    pub fn user_id(&self) -> &str {
        &self.content.user_id
    }
    pub fn ty(&self) -> &MessageEventType {
        &self.content.ty
    }
    pub fn message_id(&self) -> &str {
        &self.content.message_id
    }
    pub fn message(&self) -> &crate::Message {
        &self.content.message
    }
    pub fn alt_message(&self) -> &str {
        &self.content.alt_message
    }
    pub fn sub_type(&self) -> &str {
        &self.content.sub_type
    }
    pub fn extra(&self) -> &ExtendedMap {
        &self.content.extra
    }
}
