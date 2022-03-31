use serde::{Deserialize, Serialize};

use crate::{action::UploadFile, StandardEvent, ExtendedValue};

/// ## OneBot 12 标准动作响应
pub type Resps = Resp<RespContent>;

/// ## 动作响应
///
/// **动作响应**是 OneBot 实现收到应用端的动作请求并处理完毕后，发回应用端的数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resp<T> {
    /// 执行状态（成功与否），必须是 ok、failed 中的一个，分别表示执行成功和失败
    pub status: String,
    /// 返回码，必须符合返回码规则
    pub retcode: i64,
    /// 响应数据
    pub data: T,
    /// 错误信息，当动作执行失败时，建议在此填写人类可读的错误信息，当执行成功时，应为空字符串
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RespContent {
    SendMessage(SendMessageRespContent),
    LatestEvents(Vec<StandardEvent>),
    SupportActions(Vec<String>),
    Status(StatusContent),
    Version(VersionContent),
    UserInfo(UserInfoContent),
    FriendList(Vec<UserInfoContent>),
    GroupInfo(GroupInfoContent),
    GroupList(Vec<GroupInfoContent>),
    FileId(FileIdContent),
    PrepareFileFragmented(FileFragmentedHead),
    TransferFileFragmented(Vec<u8>),
    GetFile(UploadFile),
    Other(ExtendedValue),
}

macro_rules! resp_content {
    ($t:ty, $name: tt) => {
        impl From<$t> for RespContent {
            fn from(t: $t) -> Self {
                RespContent::$name(t)
            }
        }
    };
}

resp_content!(SendMessageRespContent, SendMessage);
resp_content!(Vec<StandardEvent>, LatestEvents);
resp_content!(Vec<String>, SupportActions);
resp_content!(StatusContent, Status);
resp_content!(VersionContent, Version);
resp_content!(UserInfoContent, UserInfo);
resp_content!(Vec<UserInfoContent>, FriendList);
resp_content!(GroupInfoContent, GroupInfo);
resp_content!(Vec<GroupInfoContent>, GroupList);
resp_content!(FileIdContent, FileId);
resp_content!(FileFragmentedHead, PrepareFileFragmented);
resp_content!(Vec<u8>, TransferFileFragmented);
resp_content!(UploadFile, GetFile);

/// ## 扩展动作响应
///
/// 已经包含标准动作响应，传 T 为扩展动作响应
///
/// 要求实现 Trait： Debug + Clone + Serialize + Deserialize + PartialEq
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExtendedRespContent<T> {
    Standard(RespContent),
    Extended(T),
}

/// 转化标准动作响应为扩展动作响应
pub trait FromStandard {
    fn from_standard(standard: RespContent) -> Self;
}

impl<T> FromStandard for ExtendedRespContent<T> {
    fn from_standard(standard: RespContent) -> Self {
        ExtendedRespContent::Standard(standard)
    }
}

impl RespContent {
    pub fn empty() -> Self {
        Self::Other(ExtendedValue::empty())
    }
}

impl FromStandard for RespContent {
    fn from_standard(standard: RespContent) -> Self {
        standard
    }
}

impl<T> Resp<T> {
    #[allow(dead_code)]
    pub fn success(data: T) -> Self {
        Resp {
            status: "ok".to_owned(),
            retcode: 0,
            data,
            message: "".to_owned(),
        }
    }

    #[allow(dead_code)]
    pub fn fail(data: T, retcode: i64, message: String) -> Self {
        Resp {
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

impl<T> Resp<T>
where
    T: FromStandard,
{
    #[allow(dead_code)]
    pub fn empty_success() -> Self {
        Self::success(T::from_standard(RespContent::empty()))
    }

    #[allow(dead_code)]
    pub fn empty_fail(retcode: i64, message: String) -> Self {
        Self::fail(T::from_standard(RespContent::empty()), retcode, message)
    }

    empty_err_resp!(bad_request, 10001, "无效的动作请求");
    empty_err_resp!(unsupported_action, 10002, "不支持的动作请求");
    empty_err_resp!(bad_param, 10003, "无效的动作请求参数");
    empty_err_resp!(unsupported_param, 10004, "不支持的动作请求参数");
    empty_err_resp!(unsupported_segment, 10005, "不支持的消息段类型");
    empty_err_resp!(bad_segment_data, 10006, "无效的消息段参数");
    empty_err_resp!(unsupported_segment_data, 10007, "不支持的消息段参数");

    empty_err_resp!(platform_error, 34000, "机器人平台错误");
}

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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SendMessageRespContent {
    pub message_id: String,
    pub time: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserInfoContent {
    pub user_id: String,
    pub nickname: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupInfoContent {
    pub group_id: String,
    pub group_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileIdContent {
    pub file_id: String,
}

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
