use serde::{Deserialize, Serialize};

/// *动作请求*是应用端为了主动向 OneBot 实现请求服务而发送的数据。
#[derive(Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum Action {
    #[serde(rename = "get_latest_events")]
    GetLatestEvents {
        limit: i64,
        timeout: i64,
    },
    #[serde(rename = "get_supported_actions")]
    GetSupportedActions,
    #[serde(rename = "get_status")]
    GetStatus,
    #[serde(rename = "get_version")]
    GetVersion,
}

#[cfg(feature = "echo")]
#[derive(Serialize, Deserialize)]
pub struct EchoAction {
    #[serde(flatten)]
    action: Action,
    echo: String,
}
