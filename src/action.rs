use serde::{Deserialize, Serialize};

// trait ActionHandleFn<T> = FnOnce(Action) -> crate::action_resp::ActionResp<T>;
// trait_alias unstable

/// *动作请求*是应用端为了主动向 OneBot 实现请求服务而发送的数据。
#[derive(Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum Action {
    // meta action
    #[serde(rename = "get_latest_events")]
    GetLatestEvents { limit: i64, timeout: i64 },
    #[serde(rename = "get_supported_actions")]
    GetSupportedActions,
    #[serde(rename = "get_status")]
    GetStatus,
    #[serde(rename = "get_version")]
    GetVersion,

    // message action
    #[serde(rename = "send_message")]
    SendMessage {
        detail_type: String,
        group_id: Option<String>,
        user_id: Option<String>,
        message: crate::Message,
    },
    #[serde(rename = "delete_message")]
    DeleteMessage { message_id: String },

    // user action
    #[serde(rename = "get_self_info")]
    GetSelfInfo,
    #[serde(rename = "get_user_info")]
    GetUserInfo { user_id: String },
    #[serde(rename = "get_friend_list")]
    GetFriendList,

    // group action
    #[serde(rename = "get_group_info")]
    GetGroupInfo { group_id: String },
    #[serde(rename = "get_group_list")]
    GetGroupList,
    #[serde(rename = "get_group_member_info")]
    GetGroupMemberInfo { group_id: String, user_id: String },
    #[serde(rename = "get_group_member_list")]
    GetGroupMemberList { group_id: String },
    #[serde(rename = "set_group_name")]
    SetGroupName {
        group_id: String,
        group_name: String,
    },
    #[serde(rename = "leave_group")]
    LeaveGroup { group_id: String },
    #[serde(rename = "kick_group_member")]
    KickGroupMember { group_id: String, user_id: String },
    #[serde(rename = "ban_group_member")]
    BanGroupMember { group_id: String, user_id: String },
    #[serde(rename = "unban_group_member")]
    UnbanGroupMember { group_id: String, user_id: String },
    #[serde(rename = "set_group_admin")]
    SetGroupAdmin { group_id: String, user_id: String },
    #[serde(rename = "unset_group_admin")]
    UnsetGroupAdmin { group_id: String, user_id: String },

    // file
    #[serde(rename = "upload_file")]
    UploadFile {
        r#type: String,
        name: String,
        url: Option<String>,
        headers: Option<std::collections::HashMap<String, String>>,
        path: Option<String>,
        data: Option<Vec<u8>>,
        sha256: Option<String>,
    },
    #[serde(rename = "upload_file_fragmented")]
    UploadFileFragmented(UploadFileFragmented),
    #[serde(rename = "get_file")]
    GetFile { file_id: String, r#type: String },
    #[serde(rename = "get_file_fragmented")]
    GetFileFragmented(GetFileFragmented),
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
