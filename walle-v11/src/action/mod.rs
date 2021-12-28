use crate::message::Message;
use serde::{Deserialize, Serialize};
mod resp;

pub use resp::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "action", content = "params", rename_all = "snake_case")]
pub enum Action {
    SendPrivateMessage {
        user_id: i32,
        message: Message,
        auto_escape: bool,
    },
    SendGroupMessage {
        group_id: i32,
        message: Message,
        auto_escape: bool,
    },
    SendMessage {
        message_type: String,
        user_id: Option<i32>,
        group_id: Option<i32>,
        message: Message,
        auto_escape: bool,
    },
    DeleteMsg {
        message_id: i32,
    },
    GetMsg {
        message_id: i32,
    },
}
