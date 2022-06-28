use serde::{Deserialize, Serialize};

use crate::action::UploadFile;
use crate::event::StandardEvent;
use crate::extra_struct;
use crate::util::{ExtendedMap, ExtendedValue, OneBotBytes};

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
    GuildInfo(GuildInfoContent),
    GuildList(Vec<GuildInfoContent>),
    ChannelInfo(ChannelInfoContent),
    ChannelList(Vec<ChannelInfoContent>),
    FileId(FileIdContent),
    PrepareFileFragmented(FileFragmentedHead),
    TransferFileFragmented(OneBotBytes),
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
        impl<E> TryFrom<RespContent<E>> for $t {
            type Error = RespContent<E>;
            fn try_from(resp: RespContent<E>) -> Result<Self, Self::Error> {
                match resp {
                    RespContent::$name(t) => Ok(t),
                    _ => Err(resp),
                }
            }
        }
        impl<E> From<Resp<$t>> for Resp<RespContent<E>> {
            fn from(resp: Resp<$t>) -> Self {
                Resp {
                    status: resp.status,
                    retcode: resp.retcode,
                    data: resp.data.into(),
                    message: resp.message,
                }
            }
        }
        impl<E> TryFrom<Resp<RespContent<E>>> for Resp<$t> {
            type Error = Resp<RespContent<E>>;
            fn try_from(resp: Resp<RespContent<E>>) -> Result<Self, Self::Error> {
                match resp {
                    Resp {
                        status,
                        retcode,
                        data: RespContent::$name(t),
                        message,
                    } => Ok(Resp {
                        status,
                        retcode,
                        data: t,
                        message,
                    }),
                    _ => Err(resp),
                }
            }
        }
    };
}

resp_content!(SendMessageRespContent, SendMessage);
resp_content!(Vec<String>, SupportActions);
resp_content!(StatusContent, Status);
resp_content!(VersionContent, Version);
resp_content!(UserInfoContent, UserInfo);
resp_content!(Vec<UserInfoContent>, FriendList);
resp_content!(GroupInfoContent, GroupInfo);
resp_content!(Vec<GroupInfoContent>, GroupList);
resp_content!(FileIdContent, FileId);
resp_content!(FileFragmentedHead, PrepareFileFragmented);
resp_content!(OneBotBytes, TransferFileFragmented);
resp_content!(UploadFile, GetFile);
resp_content!(ExtendedValue, Other);
resp_content!(GuildInfoContent, GuildInfo);
resp_content!(Vec<GuildInfoContent>, GuildList);
resp_content!(ChannelInfoContent, ChannelInfo);
resp_content!(Vec<ChannelInfoContent>, ChannelList);

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
}

pub trait RespExt {
    type Error;
    fn to_result(self) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

impl<T> RespExt for Resp<T> {
    type Error = RespError;
    fn to_result(self) -> Result<Self, RespError> {
        if self.status == "ok" {
            Ok(self)
        } else {
            Err(RespError {
                code: self.retcode,
                message: self.message,
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

extra_struct!(GuildInfoContent, guild_id: String, guild_name: String);
extra_struct!(ChannelInfoContent, channel_id: String, channel_name: String);

pub struct RespError {
    pub code: u32,
    pub message: String,
}

impl std::fmt::Debug for RespError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RespError[{}]: {}", self.code, self.message)
    }
}

impl<T> From<RespError> for Resp<T>
where
    T: From<ExtendedValue>,
{
    fn from(err: RespError) -> Self {
        Resp::fail(ExtendedValue::Null.into(), err.code, err.message)
    }
}

impl<T0, T1> From<(RespError, T0)> for Resp<T1>
where
    T1: From<T0>,
{
    fn from(err: (RespError, T0)) -> Self {
        Resp::fail(T1::from(err.1), err.0.code, err.0.message)
    }
}

pub mod resp_error {
    use super::RespError;
    /// RespError 构造函数声明
    /// ```rust
    /// error_type!(bad_request, 10001, "无效的动作请求");
    /// ```
    /// generate code:
    /// ```rust
    /// pub fn bad_request() -> RespError {
    ///     RespError {
    ///         code: 10001,
    ///         message: "无效的动作请求".to_owned(),
    ///     }
    /// }
    /// ```
    #[macro_export]
    macro_rules! error_type {
        ($name: ident, $retcode: expr, $message: expr) => {
            pub fn $name<T: std::fmt::Display>(msg: T) -> RespError {
                RespError {
                    code: $retcode,
                    message: {
                        let mut message = String::from($message);
                        let msg = msg.to_string();
                        if msg != String::default() {
                            message.push(':');
                            message.push_str(&msg);
                        }
                        message
                    },
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

    error_type!(database_error, 31000, "数据库错误");
    error_type!(filesystem_error, 32000, "文件系统错误");
    error_type!(network_error, 33000, "网络错误");
    error_type!(platform_error, 34000, "机器人平台错误");
    error_type!(tired, 36000, "I Am Tired!");
}
