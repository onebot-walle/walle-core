use serde::{Deserialize, Serialize};

/// ## OneBot 请求事件 Content
///
/// 请求事件是聊天机器人收到其他用户发送的请求对应的一类事件，例如加好友请求等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
pub enum RequestContent {
    Empty { sub_type: String },
}
