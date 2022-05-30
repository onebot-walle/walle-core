use serde::{Deserialize, Serialize};
use walle_core::ExtendedValue;

use crate::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resp {
    pub status: String,
    pub retcode: u32,
    pub data: RespContent,
}

impl Resp {
    pub fn empty_404() -> Self {
        Resp {
            status: "failed".to_string(),
            retcode: 1404,
            data: RespContent::empty(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RespContent {
    Message {
        message_id: i32,
    },
    Other(ExtendedValue),
    UserInfo {
        user_id: i64,
        nickname: String,
        sex: String,
        age: i32,
    },
    MessageDetail {
        time: i32,
        message_type: String,
        message_id: i32,
        real_id: i32,
        sender: crate::utils::Sender,
        message: Message,
    },
    NodeMessage {
        message: Vec<crate::message::Message>,
    },
}

impl RespContent {
    pub fn empty() -> Self {
        Self::Other(ExtendedValue::Null)
    }
}
