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
    /// 撤回消息
    /// 
    /// - message_id: 消息id
    /// 
    /// return None
    DeleteMsg {
        message_id: i32,
    },
    /// 获取消息
    ///
    /// - message_id: 消息id
    ///
    /// return `Resp<RespContent::MessageDetail>`
    GetMsg {
        message_id: i32,
    },
    /// 获取合并转发消息
    ///
    /// - id: 合并转发 ID
    ///
    /// return `Resp<RespContent::NodeMessage>`
    GetForwardMsg {
        id: String,
    },
    /// 发送好友赞
    ///
    /// - user_id: 对方 QQ 号
    /// - times: 赞次数，每个好友每天最多 10 次
    ///
    /// return `None`
    SendLike {
        user_id: i64,
        times: u8,
    },
}
