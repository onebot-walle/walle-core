#![doc = include_str!("README.md")]
use serde::{Deserialize, Serialize};

mod message;
mod meta;
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

pub trait EventType {
    fn event_type(&self) -> &str;
    fn detail_type(&self) -> &str;
    fn sub_type(&self) -> &str;
}

impl<T: EventType> EventType for BaseEvent<T> {
    fn event_type(&self) -> &str {
        self.content.event_type()
    }

    fn detail_type(&self) -> &str {
        self.content.detail_type()
    }

    fn sub_type(&self) -> &str {
        self.content.sub_type()
    }
}

impl EventType for EventContent {
    fn event_type(&self) -> &str {
        match self {
            EventContent::Meta(_) => "meta",
            EventContent::Message(_) => "message",
            EventContent::Notice(_) => "notice",
            EventContent::Request(_) => "request",
        }
    }

    fn detail_type(&self) -> &str {
        match self {
            EventContent::Meta(c) => c.detail_type(),
            EventContent::Message(c) => c.detail_type(),
            EventContent::Notice(c) => c.detail_type(),
            EventContent::Request(c) => c.detail_type(),
        }
    }

    fn sub_type(&self) -> &str {
        match self {
            EventContent::Meta(c) => c.sub_type(),
            EventContent::Message(c) => c.sub_type(),
            EventContent::Notice(c) => c.sub_type(),
            EventContent::Request(c) => c.sub_type(),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct RefEvent<'e, T> {
    pub id: &'e str,
    pub r#impl: &'e str,
    pub platform: &'e str,
    pub self_id: &'e str,
    pub time: f64,
    pub content: &'e T,
}

#[derive(Debug, PartialEq)]
pub struct RefMutEvent<'e, T> {
    pub id: &'e mut String,
    pub r#impl: &'e mut String,
    pub platform: &'e mut String,
    pub self_id: &'e mut String,
    pub time: f64,
    pub content: &'e mut T,
}

impl<T> BaseEvent<T> {
    pub fn as_ref<'e, T1>(&'e self) -> RefEvent<'e, T1>
    where
        T: AsRef<T1>,
    {
        RefEvent {
            id: &self.id,
            r#impl: &self.r#impl,
            platform: &self.platform,
            self_id: &self.self_id,
            time: self.time,
            content: self.content.as_ref(),
        }
    }
    pub fn as_mut<'e, T1>(&'e mut self) -> RefMutEvent<'e, T1>
    where
        T: AsMut<T1>,
    {
        RefMutEvent {
            id: &mut self.id,
            r#impl: &mut self.r#impl,
            platform: &mut self.platform,
            self_id: &mut self.self_id,
            time: self.time,
            content: self.content.as_mut(),
        }
    }
}
