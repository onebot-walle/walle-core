use crate::resp::StatusContent;
// use crate::ExtendedMap;
use serde::{Deserialize, Serialize};

/// ## OneBot 元事件 Content
///
/// 元事件是 OneBot 实现内部自发产生的一类事件，例如心跳等，
/// 与 OneBot 本身的运行状态有关，与实现对应的机器人平台无关。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "detail_type", rename_all = "snake_case")]
pub enum MetaContent {
    /// OneBot 心跳事件， OneBot 实现应每间隔 `interval` 产生一个心跳事件
    Heartbeat {
        interval: u64,
        status: StatusContent,
        sub_type: String, // just for Deserialize
                          // #[serde(flatten)]
                          // extra: ExtendedMap,
    },
}

impl super::EventType for MetaContent {
    fn event_type(&self) -> &str {
        "meta"
    }
    fn detail_type(&self) -> &str {
        match self {
            MetaContent::Heartbeat { .. } => "heartbeat",
        }
    }
    fn sub_type(&self) -> &str {
        match self {
            MetaContent::Heartbeat { sub_type, .. } => sub_type,
        }
    }
}
