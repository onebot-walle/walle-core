use serde::{Deserialize, Serialize};

use crate::{EmptyContent, Event};

pub type ActionResps = ActionResp<ActionRespContent>;

/// **动作响应**是 OneBot 实现收到应用端的动作请求并处理完毕后，发回应用端的数据。
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ActionResp<T> {
    /// 执行状态（成功与否），必须是 ok、failed 中的一个，分别表示执行成功和失败
    pub status: String,
    /// 返回码，必须符合返回码规则
    pub retcode: i64,
    /// 响应数据
    pub data: T,
    /// 错误信息，当动作执行失败时，建议在此填写人类可读的错误信息，当执行成功时，应为空字符串
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ActionRespContent {
    SendMessage(SendMessageRespContent),
    LatestEvents(Vec<Event>),
    SupportActions(Vec<String>),
    Status(StatusContent),
    Version(VersionContent),
    UserInfo(UserInfoContent),
    FriendList(Vec<UserInfoContent>),
    GroupInfo(GroupInfoContent),
    GroupList(Vec<GroupInfoContent>),
    FileId(String),
    PrepareFileFragmented(FileFragmentedHead),
    TransferFileFragmented(Vec<u8>),
    Empty(crate::action::EmptyContent), // todo
}

impl ActionRespContent {
    pub fn empty() -> Self {
        Self::Empty(crate::action::EmptyContent::default())
    }
}

impl<T> ActionResp<T> {
    #[allow(dead_code)]
    pub fn success(data: T) -> Self {
        ActionResp {
            status: "ok".to_owned(),
            retcode: 0,
            data,
            message: "".to_owned(),
        }
    }

    #[allow(dead_code)]
    pub fn fail(data: T, retcode: i64, message: String) -> Self {
        ActionResp {
            status: "failed".to_owned(),
            retcode,
            data,
            message,
        }
    }

    #[allow(dead_code)]
    pub fn tired(data: T) -> Self {
        Self::fail(data, 36000, "I Am Tired!".to_owned())
    }
}

macro_rules! empty_err_resp {
    ($fn_name: ident, $retcode: expr, $message: expr) => {
        #[allow(dead_code)]
        pub fn $fn_name() -> Self {
            Self::empty_fail($retcode, $message.to_owned())
        }
    };
}

impl ActionResp<ActionRespContent> {
    #[allow(dead_code)]
    pub fn empty_success() -> Self {
        Self::success(ActionRespContent::Empty(EmptyContent::default()))
    }

    #[allow(dead_code)]
    pub fn empty_fail(retcode: i64, message: String) -> Self {
        Self::fail(
            ActionRespContent::Empty(EmptyContent::default()),
            retcode,
            message,
        )
    }

    empty_err_resp!(bad_request, 10001, "无效的动作请求");
    empty_err_resp!(unsupported_action, 10002, "不支持的动作请求");
    empty_err_resp!(bad_param, 10003, "无效的动作请求参数");
    empty_err_resp!(unsupported_param, 10004, "不支持的动作请求参数");
    empty_err_resp!(unsupported_segment, 10005, "不支持的消息段类型");
    empty_err_resp!(bad_segment_data, 10006, "无效的消息段参数");
    empty_err_resp!(unsupported_segment_data, 10007, "不支持的消息段参数");
}

#[cfg(feature = "echo")]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct EchoActionResp<T> {
    /// 执行状态（成功与否），必须是 ok、failed 中的一个，分别表示执行成功和失败
    pub status: String,
    /// 返回码，必须符合返回码规则
    pub retcode: i64,
    /// 响应数据
    pub data: T,
    /// 错误信息，当动作执行失败时，建议在此填写人类可读的错误信息，当执行成功时，应为空字符串
    pub message: String,
    pub echo: String,
}

#[cfg(feature = "echo")]
impl<T> EchoActionResp<T> {
    pub fn new(resp: ActionResp<T>, echo: String) -> Self {
        Self {
            status: resp.status,
            retcode: resp.retcode,
            data: resp.data,
            message: resp.message,
            echo,
        }
    }

    pub fn as_action_resp(self) -> (ActionResp<T>, String) {
        (
            ActionResp {
                status: self.status,
                retcode: self.retcode,
                data: self.data,
                message: self.message,
            },
            self.echo,
        )
    }
}

// meta
pub type LatestEvents = ActionResp<Vec<Event>>;
pub type SupportActions = ActionResp<Vec<String>>;
pub type Status = ActionResp<StatusContent>;
pub type Version = ActionResp<VersionContent>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StatusContent {
    pub good: bool,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionContent {
    pub r#impl: String,
    pub platform: String,
    pub version: String,
    pub onebot_version: String,
}

impl Default for VersionContent {
    fn default() -> Self {
        VersionContent {
            r#impl: "AbrasOneBot".to_owned(),
            platform: "RustOneBot".to_owned(),
            version: "0.0.1".to_owned(),
            onebot_version: "12".to_owned(),
        }
    }
}

// message
/// Resp for send_message
pub type SendMessageResp = ActionResp<SendMessageRespContent>;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SendMessageRespContent {
    pub message_id: String,
    pub time: i64,
}

// user
/// Resp for get_self_info && get_user_info
pub type UserInfo = ActionResp<UserInfoContent>;
/// Resp for get_friend_list
pub type FriendList = ActionResp<Vec<UserInfoContent>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserInfoContent {
    pub user_id: String,
    pub nickname: String,
}

// group
/// Resp for get_group_info
pub type GroupInfo = ActionResp<GroupInfoContent>;
/// Resp for get_group_list
pub type GroupList = ActionResp<Vec<GroupInfoContent>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupInfoContent {
    pub group_id: String,
    pub group_name: String,
}

// file
/// Resp for upload_file
pub type FileId = ActionResp<String>;
/// Resp for upload_file_fragmented
pub type PrepareFileFragmented = ActionResp<FileFragmentedHead>;
/// Resp for upload_file_fragmented
pub type TransferFileFragmented = ActionResp<Vec<u8>>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileContent {
    pub name: String,
    pub url: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<Vec<u8>>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileFragmentedHead {
    pub name: String,
    pub total_size: i64,
    pub sha256: String,
}
