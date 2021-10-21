use serde::{Deserialize, Serialize};

// trait ActionHandleFn<T> = FnOnce(Action) -> crate::action_resp::ActionResp<T>;
// trait_alias unstable

/// *动作请求*是应用端为了主动向 OneBot 实现请求服务而发送的数据。
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum Action {
    // meta action
    #[serde(rename = "get_latest_events")]
    GetLatestEvents(GetLatestEventsContent),
    #[serde(rename = "get_supported_actions")]
    GetSupportedActions,
    #[serde(rename = "get_status")]
    GetStatus,
    #[serde(rename = "get_version")]
    GetVersion,

    // message action
    #[serde(rename = "send_message")]
    SendMessage(SendMessageContent),
    #[serde(rename = "delete_message")]
    DeleteMessage(DeleteMessageContent),

    // user action
    #[serde(rename = "get_self_info")]
    GetSelfInfo,
    #[serde(rename = "get_user_info")]
    GetUserInfo(UserIdContent),
    #[serde(rename = "get_friend_list")]
    GetFriendList,

    // group action
    #[serde(rename = "get_group_info")]
    GetGroupInfo(GroupIdContent),
    #[serde(rename = "get_group_list")]
    GetGroupList,
    #[serde(rename = "get_group_member_info")]
    GetGroupMemberInfo(IdsContent),
    #[serde(rename = "get_group_member_list")]
    GetGroupMemberList(GroupIdContent),
    #[serde(rename = "set_group_name")]
    SetGroupName(SetGroupNameContent),
    #[serde(rename = "leave_group")]
    LeaveGroup(GroupIdContent),
    #[serde(rename = "kick_group_member")]
    KickGroupMember(IdsContent),
    #[serde(rename = "ban_group_member")]
    BanGroupMember(IdsContent),
    #[serde(rename = "unban_group_member")]
    UnbanGroupMember(IdsContent),
    #[serde(rename = "set_group_admin")]
    SetGroupAdmin(IdsContent),
    #[serde(rename = "unset_group_admin")]
    UnsetGroupAdmin(IdsContent),

    // file
    #[serde(rename = "upload_file")]
    UploadFile(UploadFileContent),
    #[serde(rename = "upload_file_fragmented")]
    UploadFileFragmented(UploadFileFragmented),
    #[serde(rename = "get_file")]
    GetFile(GetFileContent),
    #[serde(rename = "get_file_fragmented")]
    GetFileFragmented(GetFileFragmented),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetLatestEventsContent {
    pub limit: i64,
    pub timeout: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendMessageContent {
    pub detail_type: String,
    pub group_id: Option<String>,
    pub user_id: Option<String>,
    pub message: crate::Message,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeleteMessageContent {
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserIdContent {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupIdContent {
    pub group_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdsContent {
    pub group_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetGroupNameContent {
    pub group_id: String,
    pub group_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadFileContent {
    pub r#type: String,
    pub name: String,
    pub url: Option<String>,
    pub headers:Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<Vec<u8>>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetFileContent {
    pub file_id: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "stage")]
pub enum UploadFileFragmented {
    #[serde(rename = "prepare")]
    Prepare {
        name: String,
        total: i64,
        sha256: String,
    },
    #[serde(rename = "transfer")]
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
        data: Vec<u8>,
    },
    #[serde(rename = "finish")]
    Finish { file_id: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "stage")]
pub enum GetFileFragmented {
    #[serde(rename = "prepare")]
    Prepare { file_id: String },
    #[serde(rename = "transfer")]
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
    },
}

#[cfg(feature = "echo")]
#[derive(Serialize, Deserialize)]
pub struct EchoAction {
    #[serde(flatten)]
    pub action: Action,
    pub echo: String,
}
