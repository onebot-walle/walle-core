use serde::{Deserialize, Serialize};
use walle_macro::{
    _PushToValueMap as PushToValueMap, _ToEvent as ToEvent, _TryFromEvent as TryFromEvent,
    _TryFromValue as TryFromValue,
};

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue, Hash, Serialize, Deserialize, Default,
)]
pub struct Selft {
    pub platform: String,
    pub user_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue)]
pub struct Bot {
    pub selft: Selft,
    pub online: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue, ToEvent, TryFromEvent)]
#[event(detail_type = "status_update")]
pub struct Status {
    pub good: bool,
    pub bots: Vec<Bot>,
}

#[derive(Debug, Clone, PartialEq, PushToValueMap, TryFromValue)]
pub struct SendMessageResp {
    pub message_id: String,
    pub time: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue)]
pub struct UserInfo {
    pub user_id: String,
    pub user_name: String,
    pub user_displayname: String,
    pub user_remark: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue)]
pub struct GroupInfo {
    pub group_id: String,
    pub group_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue)]
pub struct FileId {
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue)]
pub struct File {
    pub name: String,
    pub url: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<Vec<u8>>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue)]
pub struct GuildInfo {
    pub guild_id: String,
    pub guild_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue)]
pub struct ChannelInfo {
    pub channel_id: String,
    pub channel_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, TryFromValue)]
pub struct Version {
    pub implt: String,
    pub platform: String,
    pub version: String,
    pub onebot_version: String,
}
