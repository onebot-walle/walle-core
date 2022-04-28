use crate::ExtendedMap;
use serde::{Deserialize, Serialize};

/// ## OneBot 请求事件 Content
///
/// 请求事件是聊天机器人收到其他用户发送的请求对应的一类事件，例如加好友请求等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestContent {
    detail_type: String,
    sub_type: String,
    #[serde(flatten)]
    extra: ExtendedMap,
}

impl RequestContent {
    pub fn detail_type(&self) -> &str {
        &self.detail_type
    }
}
