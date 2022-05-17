use super::BaseEvent;
#[cfg(feature = "impl")]
use crate::MessageAlt;
use crate::{AsStandard, ExtendedMap};
use serde::{Deserialize, Serialize};

/// ## OneBot 消息事件 Content
///
/// 消息事件是聊天机器人收到其他用户发送的消息对应的一类事件，例如私聊消息等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageContent<D> {
    #[serde(flatten)]
    pub detail: D,
    pub message_id: String,
    pub message: crate::Message,
    pub alt_message: String,
    pub user_id: String,
    /// just for Deserialize
    pub sub_type: String,
}

/// MessageEvent detail_type ( private or group )
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "detail_type", rename_all = "snake_case")]
pub enum MessageEventDetail {
    Private {
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    Group {
        group_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
}

impl MessageEventDetail {
    pub fn group_id(&self) -> Option<&str> {
        match self {
            MessageEventDetail::Group { group_id, .. } => Some(group_id),
            _ => None,
        }
    }
}

#[cfg(feature = "impl")]
impl<D> MessageContent<D>
where
    D: From<MessageEventDetail>,
{
    pub fn new_group_message_content(
        message: crate::Message,
        message_id: String,
        user_id: String,
        group_id: String,
        extra: ExtendedMap,
    ) -> Self {
        Self {
            detail: MessageEventDetail::Group { group_id, extra }.into(),
            message_id,
            alt_message: message.alt(),
            message,
            user_id,
            sub_type: "".to_owned(),
        }
    }

    pub fn new_private_message_content(
        message: crate::Message,
        message_id: String,
        user_id: String,
        extra: ExtendedMap,
    ) -> Self {
        Self {
            detail: MessageEventDetail::Private { extra }.into(),
            message_id,
            alt_message: message.alt(),
            message,
            user_id,
            sub_type: "".to_owned(),
        }
    }
}

impl<D> BaseEvent<MessageContent<D>>
where
    D: AsStandard<MessageEventDetail>,
{
    pub fn group_id(&self) -> Option<&str> {
        self.content.detail.as_standard().group_id()
    }
    pub fn user_id(&self) -> &str {
        &self.content.user_id
    }
    pub fn detail(&self) -> &MessageEventDetail {
        &self.content.detail.as_standard()
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
        match self.content.detail.as_standard() {
            MessageEventDetail::Private { extra, .. } => extra,
            MessageEventDetail::Group { extra, .. } => extra,
        }
    }
}
