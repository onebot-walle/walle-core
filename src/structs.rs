use serde::{Deserialize, Serialize};
use walle_macro::{_OneBot as OneBot, _PushToValueMap as PushToValueMap};

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot, Hash, Serialize, Deserialize, Default,
)]
#[value]
pub struct Selft {
    pub platform: String,
    pub user_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[value]
pub struct Bot {
    pub selft: Selft,
    pub online: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[value]
#[event(detail_type = "status_update")]
pub struct Status {
    pub good: bool,
    pub bots: Vec<Bot>,
}

#[derive(Debug, Clone, PartialEq, PushToValueMap, OneBot)]
#[value]
pub struct SendMessageResp {
    pub message_id: String,
    pub time: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[value]
pub struct UserInfo {
    pub user_id: String,
    pub user_name: String,
    pub user_displayname: String,
    pub user_remark: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[value]
pub struct GroupInfo {
    pub group_id: String,
    pub group_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[value]
pub struct FileId {
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[value]
pub struct File {
    pub name: String,
    pub url: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<Vec<u8>>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[value]
pub struct GuildInfo {
    pub guild_id: String,
    pub guild_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[value]
pub struct ChannelInfo {
    pub channel_id: String,
    pub channel_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[value]
pub struct Version {
    pub implt: String,
    pub platform: String,
    pub version: String,
    pub onebot_version: String,
}
