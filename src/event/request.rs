use crate::util::ExtendedMap;
use serde::{Deserialize, Serialize};

/// ## OneBot 请求事件 Content
///
/// 请求事件是聊天机器人收到其他用户发送的请求对应的一类事件，例如加好友请求等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestContent {
    pub detail_type: String,
    pub sub_type: String,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

impl super::EventSubType for RequestContent {
    fn sub_type(&self) -> &str {
        &self.sub_type
    }
}

impl super::EventDetailType for RequestContent {
    fn detail_type(&self) -> &str {
        &self.detail_type
    }
}

impl super::EventType for RequestContent {
    fn ty(&self) -> &str {
        "request"
    }
}
