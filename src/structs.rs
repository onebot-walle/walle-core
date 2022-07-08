use crate::util::ExtendedMapExt;
use walle_macro::{_PushToMap as PushToMap, _TryFromValue as TryFromValue};

#[derive(Debug, Clone, PartialEq, Eq, PushToMap, TryFromValue)]
pub struct Status {
    pub good: bool,
    pub online: bool,
}

// impl Into<ExtendedValue> for Status {
//     fn into(self) -> ExtendedValue {
//         extended_value!({
//             "good": self.good,
//             "data": self.online,
//         })
//     }
// }

// impl TryFrom<ExtendedValue> for Status {
//     type Error = WalleError;
//     fn try_from(value: ExtendedValue) -> Result<Self, Self::Error> {
//         if let ExtendedValue::Map(mut map) = value {
//             Ok(Self {
//                 good: map.remove_downcast("good")?,
//                 online: map.remove_downcast("online")?,
//             })
//         } else {
//             Err(WalleError::ValueTypeNotMatch(
//                 "map".to_string(),
//                 format!("{:?}", value),
//             ))
//         }
//     }
// }

#[derive(Debug, Clone, PartialEq, PushToMap, TryFromValue)]
pub struct SendMessageResp {
    pub message_id: String,
    pub time: f64,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToMap, TryFromValue)]
pub struct UserInfo {
    pub user_id: String,
    pub nickname: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToMap, TryFromValue)]
pub struct GroupInfo {
    pub group_id: String,
    pub group_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToMap, TryFromValue)]
pub struct FileId {
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToMap, TryFromValue)]
pub struct File {
    pub name: String,
    pub url: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<Vec<u8>>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToMap, TryFromValue)]
pub struct GuildInfo {
    pub guild_id: String,
    pub guild_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToMap, TryFromValue)]
pub struct ChannelInfo {
    pub channel_id: String,
    pub channel_name: String,
}
