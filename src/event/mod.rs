#![doc = include_str!("README.md")]
use serde::{Deserialize, Serialize};

mod message;
mod meta;
mod next;
mod notice;
mod request;

pub use message::*;
pub use meta::*;
pub use notice::*;
pub use request::*;

use crate::util::ExtendedMap;

/// Onebot 12 标准事件
pub type StandardEvent = BaseEvent<EventContent>;
/// Onebot 12 标准消息事件
///
/// Notice: 请勿使用该类型序列化，这将导致 type 字段丢失
pub type MessageEvent = BaseEvent<MessageContent<MessageEventDetail>>;
/// Onebot 12 标准通知事件
///
/// Notice: 请勿使用该类型序列化，这将导致 type 字段丢失
pub type NoticeEvent = BaseEvent<NoticeContent>;
/// Onebot 12 标准请求事件
///
/// Notice: 请勿使用该类型序列化，这将导致 type 字段丢失
pub type RequestEvent = BaseEvent<RequestContent>;
/// Onebot 12 标准元事件
///
/// Notice: 请勿使用该类型序列化，这将导致 type 字段丢失
pub type MetaEvent = BaseEvent<MetaContent>;

/// ## OneBot Event 基类
///
/// 持有所有 Event 共有字段，其余字段由 Content 定义
///
/// *事件* 是由 OneBot 实现自发产生或从机器人平台获得，由 OneBot 实现向应用端推送的数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseEvent<T> {
    pub id: String,
    #[serde(rename = "impl")]
    pub r#impl: String,
    pub platform: String,
    pub self_id: String,
    pub time: f64,
    #[serde(flatten)]
    pub content: T,
}

/// ## Event Content
///
/// 除了 OneBot 规定的 Event 通用 Field 均为 Content
///
/// 该枚举为基础未扩展四种事件类型 Content 的枚举
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventContent {
    Meta(MetaContent),
    Message(MessageContent<MessageEventDetail>),
    Notice(NoticeContent),
    Request(RequestContent),
}

macro_rules! impl_From {
    ($from: ty, $name: tt) => {
        impl From<$from> for EventContent {
            fn from(from: $from) -> Self {
                EventContent::$name(from)
            }
        }

        impl TryFrom<EventContent> for $from {
            type Error = EventContent;

            fn try_from(content: EventContent) -> Result<Self, Self::Error> {
                match content {
                    EventContent::$name(from) => Ok(from),
                    _ => Err(content),
                }
            }
        }

        impl TryFrom<StandardEvent> for BaseEvent<$from> {
            type Error = StandardEvent;

            fn try_from(event: StandardEvent) -> Result<Self, Self::Error> {
                match event.content {
                    EventContent::$name(from) => Ok(BaseEvent {
                        id: event.id,
                        r#impl: event.r#impl,
                        platform: event.platform,
                        self_id: event.self_id,
                        time: event.time,
                        content: from,
                    }),
                    _ => Err(event),
                }
            }
        }
    };
}

impl_From!(MetaContent, Meta);
impl_From!(MessageContent<MessageEventDetail>, Message);
impl_From!(NoticeContent, Notice);
impl_From!(RequestContent, Request);

impl<T> crate::util::SelfId for BaseEvent<T> {
    fn self_id(&self) -> String {
        self.self_id.clone()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailEventContent {
    pub detail_type: String,
    #[serde(flatten)]
    pub content: ExtendedMap,
}

pub trait EventSubType {
    fn sub_type(&self) -> &str;
}

pub trait EventDetailType: EventSubType {
    fn detail_type(&self) -> &str;
}

pub trait EventType: EventDetailType {
    fn ty(&self) -> &str;
}

pub trait EventPlatform: EventType {
    fn platform(&self) -> &str;
}

pub trait EventImpl: EventPlatform {
    fn r#impl(&self) -> &str;
}

impl EventSubType for EventContent {
    fn sub_type(&self) -> &str {
        match self {
            Self::Message(message) => message.sub_type(),
            Self::Meta(meta) => meta.sub_type(),
            Self::Notice(notice) => notice.sub_type(),
            Self::Request(request) => request.sub_type(),
        }
    }
}

impl EventDetailType for EventContent {
    fn detail_type(&self) -> &str {
        match self {
            Self::Message(message) => message.detail_type(),
            Self::Meta(meta) => meta.detail_type(),
            Self::Notice(notice) => notice.detail_type(),
            Self::Request(request) => request.detail_type(),
        }
    }
}

impl EventType for EventContent {
    fn ty(&self) -> &str {
        match self {
            Self::Message(message) => message.ty(),
            Self::Meta(meta) => meta.ty(),
            Self::Notice(notice) => notice.ty(),
            Self::Request(request) => request.ty(),
        }
    }
}

impl<T: EventSubType> EventSubType for BaseEvent<T> {
    fn sub_type(&self) -> &str {
        self.content.sub_type()
    }
}

impl<T: EventDetailType> EventDetailType for BaseEvent<T> {
    fn detail_type(&self) -> &str {
        self.content.detail_type()
    }
}

impl<T: EventType> EventType for BaseEvent<T> {
    fn ty(&self) -> &str {
        self.content.ty()
    }
}

impl<T: EventType> EventPlatform for BaseEvent<T> {
    fn platform(&self) -> &str {
        &self.platform
    }
}

impl<T: EventType> EventImpl for BaseEvent<T> {
    fn r#impl(&self) -> &str {
        &self.r#impl
    }
}
