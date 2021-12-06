use crate::EmptyContent;
use serde::{Deserialize, Serialize};

/// ## OneBot 12 标准动作
///
/// **动作请求**是应用端为了主动向 OneBot 实现请求服务而发送的数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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

/// ## 扩展动作
///
/// 已经包含标准动作，传 T 为扩展动作
///
/// 要求实现 Trait： Debug + Clone + Serialize + Deserialize + PartialEq
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExtendedAction<T> {
    Standard(Action),
    Extended(T),
}

impl crate::utils::FromStandard<Action> for Action {
    fn from_standard(action: Action) -> Self {
        action
    }
}

impl<T> crate::utils::FromStandard<Action> for ExtendedAction<T> {
    fn from_standard(action: Action) -> Self {
        Self::Standard(action)
    }
}

/// Action content for GetLatestEvents
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetLatestEventsContent {
    pub limit: i64,
    pub timeout: i64,
}

/// Action content for SendMessage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SendMessageContent {
    pub detail_type: String,
    pub group_id: Option<String>,
    pub user_id: Option<String>,
    pub message: crate::Message,
}

/// Action content for DeleteMessage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeleteMessageContent {
    pub message_id: String,
}

/// Action content for UserId
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserIdContent {
    pub user_id: String,
}

/// Action content for GroupId
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GroupIdContent {
    pub group_id: String,
}

/// Action content for Ids
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IdsContent {
    pub group_id: String,
    pub user_id: String,
}

/// Action content for SetGroupName
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SetGroupNameContent {
    pub group_id: String,
    pub group_name: String,
}

/// Action content for UploadFile
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

/// Action content for GetFile
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GetFileContent {
    pub file_id: String,
    pub r#type: String,
}

/// Action content for UploadFileFragmented
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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

/// Action content for GetFileFragmented
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
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
