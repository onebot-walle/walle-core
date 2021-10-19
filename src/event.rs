use serde::{Deserialize, Serialize};

pub type Events = Event<EventContent>;
pub type MetaEvent = Event<Meta>;
pub type MessageEvent = Event<Message>;
pub type NoticeEvent = Event<Notice>;
pub type RequestEvent = Event<Request>;

/// *事件*是由 OneBot 实现自发产生或从机器人平台获得，由 OneBot 实现向应用端推送的数据。
///
/// type 为 Onebot 规定的四种事件类型，扩展事件（Extended Event）未支持。
#[derive(Serialize, Deserialize)]
pub struct Event<T> {
    pub id: String,
    #[serde(rename = "impl")]
    pub r#impl: String,
    pub platform: String,
    pub self_id: String,
    pub time: i64,
    #[serde(flatten)]
    pub content: T,
}

impl Events {
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
                time:self.time,
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
                time:self.time,
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
                time:self.time,
                content: r,
            }),
            _ => None,
        }
    }
}

/// Event Content 除了 OneBot 规定的 Event 通用 Field 均为 Content
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum EventContent {
    #[serde(rename = "meta")]
    Meta(Meta),
    #[serde(rename = "message")]
    Message(Message),
    #[serde(rename = "notice")]
    Notice(Notice),
    #[serde(rename = "request")]
    Request(Request),
}

/// OneBot 心跳事件， OneBot 实现应每间隔 `interval` 产生一个心跳事件
#[derive(Serialize, Deserialize)]
#[serde(tag = "detail_type")]
pub enum Meta {
    #[serde(rename = "heartbeat")]
    HeartBeat {
        interval: i64,
        status: crate::action_resp::StatusContent,
        sub_type: String, // just for Deserialize
    },
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    #[serde(rename = "detail_type")]
    pub ty: String, // private or group
    pub message_id: String,
    pub message: crate::Message,
    pub alt_message: String,
    pub user_id: String,
    pub group_id: Option<String>, // private MessageEvent will be None
    sub_type: String, // just for Deserialize
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "detail_type")]
pub enum Notice { 
    #[serde(rename = "group_member_increase")]
    GroupMemberIncrease {
        sub_type: String,
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    #[serde(rename = "group_member_decrease")]
    GroupMemberDecrease {
        sub_type: String,
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    #[serde(rename = "group_member_ban")]
    GroupMemberBan {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    #[serde(rename = "group_member_unban")]
    GroupMemberUnban {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    #[serde(rename = "group_admin_set")]
    GroupAdminSet {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    #[serde(rename = "group_admin_unset")]
    GroupAdminUnset {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    #[serde(rename = "group_message_delete")]
    GroupMessageDelete {
        sub_type: String,
        group_id: String,
        message_id: String,
        user_id: String,
        operator_id: String,
    },
    #[serde(rename = "friend_increase")]
    FriendIncrease {
        sub_type: String, // just for Deserialize
        user_id: String,
    },
    #[serde(rename = "friend_decrease")]
    FriendDecrease {
        sub_type: String, // just for Deserialize
        user_id: String,
    },
    #[serde(rename = "private_message_delete")]
    PrivateMessageDelete {
        sub_type: String, // just for Deserialize
        message_id: String,
        user_id: String,
    }
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "detail_type")]
pub enum Request {
    Empty {
        sub_type: String,
    }
}
