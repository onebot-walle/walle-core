use serde::{Deserialize, Serialize};
use walle_core::EmptyContent;

use crate::message::Message;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Resp {
    pub status: String,
    pub retcode: i64,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum RespContent {
    Message {
        message_id: i32,
    },
    None(EmptyContent),
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
        Self::None(EmptyContent {})
    }
}
