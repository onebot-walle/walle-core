use serde::{Deserialize, Serialize};

/// OneBot 12 标准事件
pub type Event = CustomEvent<EventContent>;
/// OneBot 12 标准元事件
pub type MetaEvent = CustomEvent<Meta>;
/// OneBot 12 标准消息事件
pub type MessageEvent = CustomEvent<Message>;
/// OneBot 12 标准通知事件
pub type NoticeEvent = CustomEvent<Notice>;
/// OneBot 12 标准请求事件
pub type RequestEvent = CustomEvent<Request>;

/// *事件*是由 OneBot 实现自发产生或从机器人平台获得，由 OneBot 实现向应用端推送的数据。
///
/// type 为 Onebot 规定的四种事件类型，扩展事件（Extended Event）未支持。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CustomEvent<T> {
    pub id: String,
    #[serde(rename = "impl")]
    pub r#impl: String,
    pub platform: String,
    pub self_id: String,
    pub time: i64,
    #[serde(flatten)]
    pub content: T,
}

impl Event {
    /// 转化为 MetaEvent
    pub fn as_meta_event(self) -> Option<MetaEvent> {
        match self.content {
            EventContent::Meta(m) => Some(MetaEvent {
                id: self.id,
                r#impl: self.r#impl,
                platform: self.platform,
                self_id: self.self_id,
                time: self.time,
                content: m,
            }),
            _ => None,
        }
    }

    /// 转化为 MessageEvent
    pub fn as_message_event(self) -> Option<MessageEvent> {
        match self.content {
            EventContent::Message(m) => Some(MessageEvent {
                id: self.id,
                r#impl: self.r#impl,
                platform: self.platform,
                self_id: self.self_id,
                time: self.time,
                content: m,
            }),
            _ => None,
        }
    }

    /// 转化为 NoticeEvent
    pub fn as_notice_event(self) -> Option<NoticeEvent> {
        match self.content {
            EventContent::Notice(n) => Some(NoticeEvent {
                id: self.id,
                r#impl: self.r#impl,
                platform: self.platform,
                self_id: self.self_id,
                time: self.time,
                content: n,
            }),
            _ => None,
        }
    }

    /// 转化为 RequestEvent
    pub fn as_request_event(self) -> Option<RequestEvent> {
        match self.content {
            EventContent::Request(r) => Some(RequestEvent {
                id: self.id,
                r#impl: self.r#impl,
                platform: self.platform,
                self_id: self.self_id,
                time: self.time,
                content: r,
            }),
            _ => None,
        }
    }
}

/// Event Content 除了 OneBot 规定的 Event 通用 Field 均为 Content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type")]
#[serde(rename_all = "snake_case")]
pub enum EventContent {
    Meta(Meta),
    Message(Message),
    Notice(Notice),
    Request(Request),
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
        Self::Message(Message {
            ty,
            message_id,
            message,
            alt_message,
            user_id,
            sub_type: "".to_owned(),
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

/// OneBot 元事件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "snake_case")]
pub enum Meta {
    /// OneBot 心跳事件， OneBot 实现应每间隔 `interval` 产生一个心跳事件
    Heartbeat {
        interval: i64,
        status: crate::action_resp::StatusContent,
        sub_type: String, // just for Deserialize
    },
}

/// OneBot 消息事件
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    #[serde(flatten)]
    pub ty: MessageEventType,
    pub message_id: String,
    pub message: crate::Message,
    pub alt_message: String,
    pub user_id: String,
    pub(crate) sub_type: String, // just for Deserialize
}

/// MessageEvent detail_type ( private or group )
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "snake_case")]
pub enum MessageEventType {
    Private,
    Group { group_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "snake_case")]
pub enum Notice {
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
pub enum Request {
    Empty { sub_type: String },
}
