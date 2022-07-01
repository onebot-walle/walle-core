use crate::util::{ExtendedMap, ExtendedMapExt, OneBotBytes, SelfId};
use crate::{extra_struct, message::MSVistor};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

#[cfg(feature = "app")]
mod ext;
#[cfg(feature = "app")]
pub use ext::*;

/// ## OneBot 12 标准动作
///
/// **动作请求**是应用端为了主动向 OneBot 实现请求服务而发送的数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "params", rename_all = "snake_case")]
pub enum StandardAction {
    // meta action
    GetLatestEvents(GetLatestEvents),
    GetSupportedActions(ExtendedMap),
    GetStatus(ExtendedMap),
    GetVersion(ExtendedMap),

    // message action
    SendMessage(SendMessage),
    DeleteMessage(DeleteMessage),

    // user action
    GetSelfInfo(ExtendedMap),
    GetUserInfo(GetUserInfo),
    GetFriendList(ExtendedMap),

    // group action
    GetGroupInfo(GetGroupInfo),
    GetGroupList(ExtendedMap),
    GetGroupMemberInfo(GetGroupMemberInfo),
    GetGroupMemberList(GetGroupMemberList),
    SetGroupName(SetGroupName),
    LeaveGroup(LeaveGroup),

    // guild action
    GetGuildInfo(GetGuildInfo),
    GetGuildList(ExtendedMap),
    GetChannelInfo(GetChannelInfo),
    GetChannelList(GetChannelList),
    GetGuildMemberInfo(GetGuildMemberInfo),
    GetGuildMemberList(GetGuildMemberList),
    SetGuildName(SetGuildName),
    SetChannelName(SetChannelName),
    LeaveGuild(LeaveGuild),

    // file
    UploadFile(UploadFile),
    UploadFileFragmented(UploadFileFragmented),
    GetFile(GetFile),
    GetFileFragmented(GetFileFragmented),
}

/// OneBot 12 扩展动作
///
/// 任何符合 OneBot 12 格式的动作均可序列化为该 struct
///
/// 如果需要使用该动作，可以使用 untagged enum 包裹该动作：
///
/// ```rust
/// use onebot_12::{StandardAction, ExtendedAction};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// #[serde(untagged)]
/// pub enum YourAction {
///     Standard(StandardAction),
///     Extended(ExtendedAction),
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtendedAction {
    pub action: String,
    pub params: ExtendedMap,
}

impl StandardAction {
    pub fn action_type(&self) -> &str {
        match self {
            Self::GetLatestEvents(_) => "get_latest_events",
            Self::GetSupportedActions(_) => "get_supported_actions",
            Self::GetStatus(_) => "get_status",
            Self::GetVersion(_) => "get_version",
            Self::SendMessage(_) => "send_message",
            Self::DeleteMessage(_) => "delete_message",
            Self::GetSelfInfo(_) => "get_self_info",
            Self::GetUserInfo(_) => "get_user_info",
            Self::GetFriendList(_) => "get_friend_list",
            Self::GetGroupInfo(_) => "get_group_info",
            Self::GetGroupList(_) => "get_group_list",
            Self::GetGroupMemberInfo(_) => "get_group_member_info",
            Self::GetGroupMemberList(_) => "get_group_member_list",
            Self::SetGroupName(_) => "set_group_name",
            Self::LeaveGroup(_) => "leave_group",
            Self::GetGuildInfo(_) => "get_guild_info",
            Self::GetGuildList(_) => "get_guild_list",
            Self::GetChannelInfo(_) => "get_channel_info",
            Self::GetChannelList(_) => "get_channel_list",
            Self::GetGuildMemberInfo(_) => "get_guild_member_info",
            Self::GetGuildMemberList(_) => "get_guild_member_list",
            Self::SetGuildName(_) => "set_guild_name",
            Self::SetChannelName(_) => "set_channel_name",
            Self::LeaveGuild(_) => "leave_guild",
            Self::UploadFile(_) => "upload_file",
            Self::UploadFileFragmented(_) => "upload_file_fragmented",
            Self::GetFile(_) => "get_file",
            Self::GetFileFragmented(_) => "get_file_fragmented",
        }
    }
}

macro_rules! impl_from(
    ($to:ident => $($sub: ident),*) => {
        $(impl From<$sub> for $to {
            fn from(from: $sub) -> Self {
                $to::$sub(from)
            }
        })*
    };
);

/// Action content for SendMessage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SendMessage {
    pub detail_type: String,
    pub group_id: Option<String>,
    pub user_id: Option<String>,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    #[serde(deserialize_with = "deserialize_message")]
    pub message: crate::message::Message,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

struct MessageVisitor;

impl<'de> Visitor<'de> for MessageVisitor {
    type Value = crate::message::Message;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a message")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut message = Vec::new();
        while let Some(segment) = seq.next_element()? {
            message.push(segment);
        }
        Ok(message)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        MSVistor::_visit_map(map).map(|s| vec![s])
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(vec![crate::message::MessageSegment::text(s.to_owned())])
    }
}

fn deserialize_message<'de, D>(deserializer: D) -> Result<crate::message::Message, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(MessageVisitor)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetLatestEvents {
    #[serde(default)]
    pub limit: i64,
    #[serde(default)]
    pub timeout: i64,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}
// onebot_action!(GetLatestEvents, limit: i64, timeout: i64);
extra_struct!(DeleteMessage, message_id: String);
extra_struct!(GetUserInfo, user_id: String);
extra_struct!(GetGroupInfo, group_id: String);
extra_struct!(GetGroupMemberList, group_id: String);
extra_struct!(LeaveGroup, group_id: String);
extra_struct!(GetGroupMemberInfo, group_id: String, user_id: String);
extra_struct!(SetGroupName, group_id: String, group_name: String);
extra_struct!(GetChannelInfo, guild_id: String, channel_id: String);
extra_struct!(GetChannelList, guild_id: String);
extra_struct!(GetGuildMemberInfo, guild_id: String, user_id: String);
extra_struct!(GetGuildMemberList, guild_id: String);
extra_struct!(SetGuildName, guild_id: String, guild_name: String);
extra_struct!(
    SetChannelName,
    guild_id: String,
    channel_id: String,
    channel_name: String
);
extra_struct!(LeaveGuild, guild_id: String);
extra_struct!(
    UploadFile,
    r#type: String,
    name: String,
    url: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
    path: Option<String>,
    data: Option<OneBotBytes>,
    sha256: Option<String>
);
extra_struct!(GetFile, file_id: String, r#type: String);
extra_struct!(GetGuildInfo, guild_id: String);

/// Action content for UploadFileFragmented
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum UploadFileFragmented {
    Prepare {
        name: String,
        total_size: i64,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
        data: OneBotBytes,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    Finish {
        file_id: String,
        sha256: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
}

/// Action content for GetFileFragmented
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum GetFileFragmented {
    Prepare {
        file_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
}

impl_from!(
    StandardAction =>
    SendMessage,
    GetLatestEvents,
    DeleteMessage,
    GetUserInfo,
    GetGroupInfo,
    GetGroupMemberList,
    LeaveGroup,
    GetGroupMemberInfo,
    SetGroupName,
    UploadFile,
    UploadFileFragmented,
    GetFile,
    GetFileFragmented
);

pub trait ActionType {
    fn content_type(&self) -> crate::util::ContentType;
}

impl ActionType for StandardAction {
    fn content_type(&self) -> crate::util::ContentType {
        match self {
            Self::UploadFile(_)
            | Self::UploadFileFragmented(_)
            | Self::GetFile(_)
            | Self::GetFileFragmented(_) => crate::util::ContentType::MsgPack,
            _ => crate::util::ContentType::Json,
        }
    }
}

impl SelfId for StandardAction {
    fn self_id(&self) -> String {
        match self {
            StandardAction::GetLatestEvents(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::GetSupportedActions(g) => g.try_get("self_id").unwrap_or_default(),
            StandardAction::GetStatus(g) => g.try_get("self_id").unwrap_or_default(),
            StandardAction::GetVersion(g) => g.try_get("self_id").unwrap_or_default(),

            StandardAction::SendMessage(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::DeleteMessage(g) => g.extra.try_get("self_id").unwrap_or_default(),

            StandardAction::GetSelfInfo(g) => g.try_get("self_id").unwrap_or_default(),
            StandardAction::GetUserInfo(g) => g.extra.try_get("self_id").unwrap_or_default(),

            StandardAction::GetGroupInfo(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::GetGroupList(g) => g.try_get("self_id").unwrap_or_default(),
            StandardAction::GetGroupMemberInfo(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::GetGroupMemberList(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::SetGroupName(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::LeaveGroup(g) => g.extra.try_get("self_id").unwrap_or_default(),

            StandardAction::GetGuildInfo(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::GetGuildList(g) => g.try_get("self_id").unwrap_or_default(),
            StandardAction::GetChannelInfo(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::GetChannelList(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::GetGuildMemberInfo(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::GetGuildMemberList(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::SetGuildName(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::SetChannelName(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::LeaveGuild(g) => g.extra.try_get("self_id").unwrap_or_default(),

            StandardAction::UploadFile(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::UploadFileFragmented(g) => g.self_id(),
            StandardAction::GetFile(g) => g.extra.try_get("self_id").unwrap_or_default(),
            StandardAction::GetFileFragmented(g) => g.self_id(),
            StandardAction::GetFriendList(g) => g.try_get("self_id").unwrap_or_default(),
        }
    }
}

impl SelfId for UploadFileFragmented {
    fn self_id(&self) -> String {
        match self {
            Self::Prepare { extra, .. } => extra.try_get("self_id").unwrap_or_default(),
            Self::Transfer { extra, .. } => extra.try_get("self_id").unwrap_or_default(),
            Self::Finish { extra, .. } => extra.try_get("self_id").unwrap_or_default(),
        }
    }
}

impl SelfId for GetFileFragmented {
    fn self_id(&self) -> String {
        match self {
            Self::Prepare { extra, .. } => extra.try_get("self_id").unwrap_or_default(),
            Self::Transfer { extra, .. } => extra.try_get("self_id").unwrap_or_default(),
        }
    }
}

#[test]
fn send_test() {
    let send_json = r#"{
        "action": "send_message",
        "params": {
            "detail_type": "private",
            "user_id": "user_id",
            "group_id": "group_id",
            "message": [
                {
                    "type": "text",
                    "data": {
                        "text": "test"
                    }
                }
            ]
        }
    }"#;
    let action: StandardAction = serde_json::from_str(send_json).unwrap();
    println!("{:?}", action);
}
