use crate::utils::ExtendedMap;
use serde::{Deserialize, Serialize};

mod to;

pub trait To12 {
    fn to_12(self) -> Result<walle_core::Event, &'static str>;
}

pub trait To11 {
    fn to_11(self) -> Result<Event, &'static str>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub time: i64,
    pub self_id: String,
    #[serde(flatten)]
    pub content: EventContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "post_type")]
#[serde(rename_all = "snake_case")]
pub enum EventContent {
    Message(Message),
    Notice(Notice),
    Request(Request),
    MetaEvent(MetaEvent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub message_id: i32,
    pub user_id: i64,
    pub message: crate::message::Message,
    pub raw_message: String,
    pub font: i32,
    #[serde(flatten)]
    pub sub: MessageSub,
    #[serde(flatten)]
    pub extend_data: ExtendedMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "post_type")]
#[serde(rename_all = "snake_case")]
pub enum MessageSub {
    Private {
        sub_type: String,
        sender: crate::utils::PrivateSender,
    },
    Group {
        group_id: i64,
        sender: crate::utils::GroupSender,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "notice_type")]
#[serde(rename_all = "snake_case")]
pub enum Notice {
    GroupUpload {
        group_id: i64,
        user_id: i64,
        file: File,
    },
    GroupAdmin {
        sub_type: String,
        group_id: i64,
        user_id: i64,
    },
    GroupDecrease {
        sub_type: String,
        group_id: i64,
        operator_id: i64,
        user_id: i64,
    },
    GroupIncrease {
        sub_type: String,
        group_id: i64,
        operator_id: i64,
        user_id: i64,
    },
    GroupBan {
        sub_type: String,
        group_id: i64,
        operator_id: i64,
        user_id: i64,
        duration: i64,
    },
    FriendAdd {
        user_id: i64,
    },
    GroupRecall {
        group_id: i64,
        user_id: i64,
        operator_id: i64,
        message_id: i64,
    },
    FriendRecall {
        user_id: i64,
        message_id: i64,
    },
    Notify(Notify),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "sub_type")]
#[serde(rename_all = "snake_case")]
pub enum Notify {
    Poke {
        group_id: i64,
        user_id: i64,
        target_id: i64,
    },
    LuckyKing {
        group_id: i64,
        user_id: i64,
        target_id: i64,
    },
    Honor {
        group_id: i64,
        honor_type: String,
        user_id: i64,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct File {
    pub id: String,
    pub name: String,
    pub size: i64,
    pub busid: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "request_type")]
#[serde(rename_all = "snake_case")]
pub enum Request {
    Friend {
        user_id: i64,
        comment: String,
        flag: String,
    },
    Group {
        sub_type: String,
        group_id: i64,
        user_id: i64,
        comment: String,
        flag: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "meta_event_type")]
#[serde(rename_all = "snake_case")]
pub enum MetaEvent {
    Lifecycle {
        sub_type: String,
    },
    Heartbeat {
        status: walle_core::action_resp::StatusContent,
        interval: i64,
    },
}
