use serde::{Deserialize, Serialize};

use crate::{action::UploadFile, ExtendedMap, ExtendedValue, StandardEvent};

/// ## OneBot 12 标准动作响应
pub type StandardResps = Resps<StandardEvent>;
pub type StandardRespContent = RespContent<StandardEvent>;
pub type Resps<E> = Resp<RespContent<E>>;

/// ## 动作响应
///
/// **动作响应**是 OneBot 实现收到应用端的动作请求并处理完毕后，发回应用端的数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resp<T> {
    /// 执行状态（成功与否），必须是 ok、failed 中的一个，分别表示执行成功和失败
    pub status: String,
    /// 返回码，必须符合返回码规则
    pub retcode: u32,
    /// 响应数据
    pub data: T,
    /// 错误信息，当动作执行失败时，建议在此填写人类可读的错误信息，当执行成功时，应为空字符串
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum RespContent<E> {
    SendMessage(SendMessageRespContent),
    LatestEvents(Vec<E>),
    SupportActions(Vec<String>),
    Status(StatusContent),
    Version(VersionContent),
    MessageEvent(E),
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
        impl<E> From<$t> for RespContent<E> {
            fn from(t: $t) -> Self {
                RespContent::$name(t)
            }
        }
    };
}

resp_content!(SendMessageRespContent, SendMessage);
// resp_content!(Vec<StandardEvent>, LatestEvents);
resp_content!(Vec<String>, SupportActions);
resp_content!(StatusContent, Status);
resp_content!(VersionContent, Version);
// resp_content!(StandardEvent, MessageEvent);
resp_content!(UserInfoContent, UserInfo);
resp_content!(Vec<UserInfoContent>, FriendList);
resp_content!(GroupInfoContent, GroupInfo);
resp_content!(Vec<GroupInfoContent>, GroupList);
resp_content!(FileIdContent, FileId);
resp_content!(FileFragmentedHead, PrepareFileFragmented);
resp_content!(Vec<u8>, TransferFileFragmented);
resp_content!(UploadFile, GetFile);
resp_content!(ExtendedValue, Other);

impl<T> Resp<T> {
    #[allow(dead_code)]
    pub fn success(data: T) -> Self {
        Resp {
            status: "ok".to_string(),
            retcode: 0,
            data,
            message: "".to_owned(),
        }
    }

    #[allow(dead_code)]
    pub fn fail(data: T, retcode: u32, message: String) -> Self {
        Resp {
            status: "failed".to_string(),
            retcode,
            data,
            message,
        }
    }
}

impl<T> Resp<T>
where
    T: From<ExtendedValue>,
{
    #[allow(dead_code)]
    pub fn empty_success() -> Self {
        Self::success(T::from(ExtendedValue::empty_map()))
    }

    #[allow(dead_code)]
    pub fn empty_fail(retcode: u32, message: String) -> Self {
        Self::fail(T::from(ExtendedValue::empty_map()), retcode, message)
    }

    pub fn as_result(self) -> Result<Self, RespError<T>> {
        if self.status == "ok" {
            Ok(self)
        } else {
            Err(RespError {
                code: self.retcode,
                message: self.message,
                data: self.data,
            })
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusContent {
    pub good: bool,
    pub online: bool,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionContent {
    pub r#impl: String,
    pub platform: String,
    pub version: String,
    pub onebot_version: String,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

impl Default for VersionContent {
    fn default() -> Self {
        VersionContent {
            r#impl: "Walle".to_owned(),
            platform: "RustOneBot".to_owned(),
            version: "0.0.1".to_owned(),
            onebot_version: "12".to_owned(),
            extra: ExtendedMap::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SendMessageRespContent {
    pub message_id: String,
    pub time: f64,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserInfoContent {
    pub user_id: String,
    pub nickname: String,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupInfoContent {
    pub group_id: String,
    pub group_name: String,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileIdContent {
    pub file_id: String,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileContent {
    pub name: String,
    pub url: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<Vec<u8>>,
    pub sha256: Option<String>,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileFragmentedHead {
    pub name: String,
    pub total_size: i64,
    pub sha256: String,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

pub struct RespError<T> {
    pub code: u32,
    pub message: String,
    pub data: T,
}

impl<T> From<RespError<T>> for Resp<T> {
    fn from(err: RespError<T>) -> Self {
        Resp::fail(err.data.into(), err.code, err.message)
    }
}

pub mod resp_error_builder {
    use super::RespError;
    use crate::ExtendedValue;
    #[macro_export]
    macro_rules! error_type {
        ($name: ident, $retcode: expr, $message: expr) => {
            pub fn $name<T>() -> RespError<T>
            where
                T: From<ExtendedValue>,
            {
                RespError {
                    code: $retcode,
                    message: $message.to_owned(),
                    data: ExtendedValue::Null.into(),
                }
            }
        };
    }

    error_type!(bad_request, 10001, "无效的动作请求");
    error_type!(unsupported_action, 10002, "不支持的动作");
    error_type!(bad_param, 10003, "无效的动作请求参数");
    error_type!(unsupported_param, 10004, "不支持的动作请求参数");
    error_type!(unsupported_segment, 10005, "不支持的消息段类型");
    error_type!(bad_segment_data, 10006, "无效的消息段参数");
    error_type!(unsupported_segment_data, 10007, "不支持的消息段参数");

    error_type!(bad_handler, 20001, "动作处理器实现错误");
    error_type!(internal_handler, 20002, "动作处理器运行时抛出异常");

    error_type!(tired, 36000, "I Am Tired!");
}
