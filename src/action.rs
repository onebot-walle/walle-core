use serde::{Deserialize, Serialize};

// trait ActionHandleFn<T> = FnOnce(Action) -> crate::action_resp::ActionResp<T>;
// trait_alias unstable

/// 空结构体，用于对应 Json 中的空 Map
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct EmptyContent {}

impl Default for EmptyContent {
    fn default() -> Self {
        Self {}
    }
}

/// **动作请求**是应用端为了主动向 OneBot 实现请求服务而发送的数据。
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "params")]
#[serde(rename_all = "snake_case")]
pub enum Action {
    // meta action
    GetLatestEvents(GetLatestEventsContent),
    GetSupportedActions(EmptyContent),
    GetStatus(EmptyContent),
    GetVersion(EmptyContent),

    // message action
    SendMessage(SendMessageContent),
    DeleteMessage(DeleteMessageContent),

    // user action
    GetSelfInfo(EmptyContent),
    GetUserInfo(UserIdContent),
    GetFriendList(EmptyContent),

    // group action
    GetGroupInfo(GroupIdContent),
    GetGroupList(EmptyContent),
    GetGroupMemberInfo(IdsContent),
    GetGroupMemberList(GroupIdContent),
    SetGroupName(SetGroupNameContent),
    LeaveGroup(GroupIdContent),
    KickGroupMember(IdsContent),
    BanGroupMember(IdsContent),
    UnbanGroupMember(IdsContent),
    SetGroupAdmin(IdsContent),
    UnsetGroupAdmin(IdsContent),

    // file
    UploadFile(UploadFileContent),
    UploadFileFragmented(UploadFileFragmented),
    GetFile(GetFileContent),
    GetFileFragmented(GetFileFragmented),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetLatestEventsContent {
    pub limit: i64,
    pub timeout: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SendMessageContent {
    pub detail_type: String,
    pub group_id: Option<String>,
    pub user_id: Option<String>,
    pub message: crate::Message,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteMessageContent {
    pub message_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserIdContent {
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupIdContent {
    pub group_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdsContent {
    pub group_id: String,
    pub user_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetGroupNameContent {
    pub group_id: String,
    pub group_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UploadFileContent {
    pub r#type: String,
    pub name: String,
    pub url: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<Vec<u8>>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetFileContent {
    pub file_id: String,
    pub r#type: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "stage")]
#[serde(rename_all = "snake_case")]
pub enum UploadFileFragmented {
    Prepare {
        name: String,
        total: i64,
        sha256: String,
    },
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
        data: Vec<u8>,
    },
    Finish {
        file_id: String,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "stage")]
#[serde(rename_all = "snake_case")]
pub enum GetFileFragmented {
    Prepare {
        file_id: String,
    },
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
    },
}

#[cfg(feature = "echo")]
#[derive(Serialize, Deserialize, PartialEq)]
pub struct EchoAction {
    #[serde(flatten)]
    pub action: Action,
    pub echo: String,
}
