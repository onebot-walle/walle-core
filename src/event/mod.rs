#![doc = include_str!("README.md")]
use serde::{Deserialize, Serialize};

use crate::ExtendedMap;
#[cfg(feature = "impl")]
use crate::MessageAlt;

/// OneBot 12 标准事件
pub type Event = BaseEvent<EventContent>;

/// ## OneBot Event 基类
///
/// 持有所有 Event 共有字段，其余字段由 Content 定义
///
/// **事件**是由 OneBot 实现自发产生或从机器人平台获得，由 OneBot 实现向应用端推送的数据。
///
/// type 为 Onebot 规定的四种事件类型，扩展事件（Extended Event）未支持。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BaseEvent<T> {
    pub id: String,
    #[serde(rename = "impl")]
    pub r#impl: String,
    pub platform: String,
    pub self_id: String,
    pub time: u64,
    #[serde(flatten)]
    pub content: T,
}

/// ## Event Content
///
/// 除了 OneBot 规定的 Event 通用 Field 均为 Content
///
/// 该枚举为基础未扩展四种事件类型 Content 的枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EventContent {
    Meta(MetaContent),
    Message(MessageContent),
    Notice(NoticeContent),
    Request(RequestContent),
}

macro_rules! impl_from {
    ($from: ty, $name: tt) => {
        impl From<$from> for EventContent {
            fn from(from: $from) -> Self {
                EventContent::$name(from)
            }
        }
    };
}

impl_from!(MetaContent, Meta);
impl_from!(MessageContent, Message);
impl_from!(NoticeContent, Notice);
impl_from!(RequestContent, Request);

impl crate::utils::FromStandard<EventContent> for EventContent {
    fn from_standard(event_content: EventContent) -> Self {
        event_content
    }
}

impl EventContent {
    /// build a new MessageContent
    pub fn new_message_content(
        ty: MessageEventType,
        message_id: String,
        message: crate::message::Message,
        alt_message: String,
        user_id: String,
    ) -> Self {
        Self::Message(MessageContent {
            ty,
            message_id,
            message,
            alt_message,
            user_id,
            sub_type: "".to_owned(),
            extra: ExtendedMap::default(),
        })
    }

    /// build a new MessageContent with Private type
    pub fn private(
        message_id: String,
        message: crate::message::Message,
        alt_message: String,
        user_id: String,
    ) -> Self {
        Self::new_message_content(
            MessageEventType::Private,
            message_id,
            message,
            alt_message,
            user_id,
        )
    }

    /// build a new MessageContent with Group type
    pub fn group(
        message_id: String,
        message: crate::message::Message,
        alt_message: String,
        user_id: String,
        group_id: String,
    ) -> Self {
        Self::new_message_content(
            MessageEventType::Group { group_id },
            message_id,
            message,
            alt_message,
            user_id,
        )
    }
}

/// ## OneBot 元事件 Content
///
/// 元事件是 OneBot 实现内部自发产生的一类事件，例如心跳等，
/// 与 OneBot 本身的运行状态有关，与实现对应的机器人平台无关。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "snake_case")]
pub enum MetaContent {
    /// OneBot 心跳事件， OneBot 实现应每间隔 `interval` 产生一个心跳事件
    Heartbeat {
        interval: u32,
        status: crate::resp::StatusContent,
        sub_type: String, // just for Deserialize
    },
}

impl MetaContent {
    pub fn detail_type(&self) -> &str {
        match self {
            MetaContent::Heartbeat { .. } => "Heartbeat",
        }
    }
}

/// ## 扩展元事件
///
/// 已经包含标准事件，传 T 为扩展事件
///
/// 要求实现 Trait： Debug + Clone + Serialize + Deserialize + PartialEq
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExtendedMeta<T> {
    Standard(MetaContent),
    Extended(T),
}

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

#[cfg(feature = "impl")]
impl MessageContent {
    pub fn new_group_message_content(
        message: crate::Message,
        user_id: String,
        group_id: String,
        extra: ExtendedMap,
    ) -> Self {
        Self {
            ty: MessageEventType::Group { group_id },
            message_id: crate::utils::new_uuid(),
            alt_message: message.alt(),
            message,
            user_id,
            sub_type: "".to_owned(),
            extra,
        }
    }

    pub fn new_private_message_content(
        message: crate::Message,
        user_id: String,
        extra: ExtendedMap,
    ) -> Self {
        Self {
            ty: MessageEventType::Private,
            message_id: crate::utils::new_uuid(),
            alt_message: message.alt(),
            message,
            user_id,
            sub_type: "".to_owned(),
            extra,
        }
    }
}

/// ## 扩展消息事件
///
/// 已经包含标准事件，传 T 为扩展事件
///
/// 要求实现 Trait： Debug + Clone + Serialize + Deserialize + PartialEq
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExtendedMessage<T> {
    Standard(MessageContent),
    Extended(T),
}

/// MessageEvent detail_type ( private or group )
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "snake_case")]
pub enum MessageEventType {
    Private,
    Group { group_id: String },
}

/// ## OneBot 通知事件 Content
///
/// 通知事件是机器人平台向机器人发送通知对应的事件，例如群成员变动等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "snake_case")]
pub enum NoticeContent {
    GroupMemberIncrease {
        sub_type: String,
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupMemberDecrease {
        sub_type: String,
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupMemberBan {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupMemberUnban {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupAdminSet {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupAdminUnset {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupMessageDelete {
        sub_type: String,
        group_id: String,
        message_id: String,
        user_id: String,
        operator_id: String,
    },
    FriendIncrease {
        sub_type: String, // just for Deserialize
        user_id: String,
    },
    FriendDecrease {
        sub_type: String, // just for Deserialize
        user_id: String,
    },
    PrivateMessageDelete {
        sub_type: String, // just for Deserialize
        message_id: String,
        user_id: String,
    },
}

/// ## 扩展通知事件
///
/// 已经包含标准事件，传 T 为扩展事件
///
/// 要求实现 Trait： Debug + Clone + Serialize + Deserialize + PartialEq
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ExtendedNotice<T> {
    Standard(NoticeContent),
    Extended(T),
}

/// ## OneBot 请求事件 Content
///
/// 请求事件是聊天机器人收到其他用户发送的请求对应的一类事件，例如加好友请求等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
pub enum RequestContent {
    Empty { sub_type: String },
}

/// ## 扩展请求事件
///
/// 已经包含标准事件，传 T 为扩展事件
///
/// 要求实现 Trait： Debug + Clone + Serialize + Deserialize + PartialEq
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum ExtendedRequest<T> {
    Standard(RequestContent),
    Extended(T),
}

#[cfg(feature = "impl")]
#[cfg_attr(docsrs, doc(cfg(feature = "impl")))]
#[async_trait::async_trait]
impl crate::HeartbeatBuild for Event {
    async fn build_heartbeat<A, R, const V: u8>(
        ob: &crate::impls::CustomOneBot<Self, A, R, V>,
        interval: u32,
    ) -> Self {
        ob.new_event(EventContent::Meta(MetaContent::Heartbeat {
            interval,
            status: ob.get_status(),
            sub_type: "".to_string(),
        }))
        .await
    }
}

impl<T> crate::BasicEvent for BaseEvent<T> {
    fn self_id(&self) -> String {
        self.self_id.clone()
    }
}
