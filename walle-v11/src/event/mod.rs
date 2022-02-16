use serde::{Deserialize, Serialize};
use walle_core::{BasicEvent, ExtendedMap, HeartbeatBuild};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub time: u64,
    pub self_id: i64,
    #[serde(flatten)]
    pub content: EventContent,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "post_type")]
#[serde(rename_all = "snake_case")]
pub enum EventContent {
    Message(MessageContent),
    Notice(NoticeContent),
    Request(RequestContent),
    MetaEvent(MetaContent),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MessageContent {
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
#[serde(tag = "message_type")]
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
pub enum NoticeContent {
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
pub enum RequestContent {
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
pub enum MetaContent {
    Lifecycle {
        sub_type: String,
    },
    Heartbeat {
        status: walle_core::resp::StatusContent,
        interval: i64,
    },
}

impl MetaContent {
    pub fn detail_type(&self) -> &str {
        match self {
            Self::Lifecycle { .. } => "lifecycle",
            Self::Heartbeat { .. } => "heartbeat",
        }
    }
}

impl BasicEvent for Event {
    fn self_id(&self) -> String {
        self.self_id.to_string()
    }
}

macro_rules! impl_from {
    ($sub: tt, $sub_ty: ty) => {
        impl From<$sub_ty> for EventContent {
            fn from(sub: $sub_ty) -> Self {
                EventContent::$sub(sub)
            }
        }
    };
}

impl_from!(Message, MessageContent);
impl_from!(Notice, NoticeContent);
impl_from!(Request, RequestContent);
impl_from!(MetaEvent, MetaContent);

#[async_trait::async_trait]
impl HeartbeatBuild for Event {
    async fn build_heartbeat<A, R, const V: u8>(
        ob: &walle_core::impls::CustomOneBot<Event, A, R, V>,
        interval: u32,
    ) -> Self {
        Event {
            time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            self_id: ob.self_id().await.parse().unwrap(),
            content: EventContent::MetaEvent(MetaContent::Heartbeat {
                status: ob.get_status(),
                interval: interval as i64,
            }),
        }
    }
}
