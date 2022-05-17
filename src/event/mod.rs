#![doc = include_str!("README.md")]
use serde::{Deserialize, Serialize};

mod message;
pub use message::*;
mod notice;
pub use notice::*;
mod meta;
pub use meta::*;
mod request;
pub use request::*;

use crate::ExtendedMap;

/// OneBot 12 标准事件
pub type StandardEvent = BaseEvent<EventContent>;
pub type MessageEvent = BaseEvent<MessageContent<MessageEventDetail>>;
pub type NoticeEvent = BaseEvent<NoticeContent>;
pub type RequestEvent = BaseEvent<RequestContent>;
pub type MetaEvent = BaseEvent<MetaContent>;

/// ## OneBot Event 基类
///
/// 持有所有 Event 共有字段，其余字段由 Content 定义
///
/// **事件**是由 OneBot 实现自发产生或从机器人平台获得，由 OneBot 实现向应用端推送的数据。
///
/// type 为 Onebot 规定的四种事件类型，扩展事件（Extended Event）未支持。
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

impl<T> crate::SelfId for BaseEvent<T> {
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
