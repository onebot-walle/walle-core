//! ActionResp 相关模型定义

use std::fmt::Display;

use serde::{Deserialize, Serialize};

use crate::{
    prelude::{WalleError, WalleResult},
    util::Value,
};

/// ActionResp 通用模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resp {
    /// 执行状态（成功与否），必须是 ok、failed 中的一个，分别表示执行成功和失败
    pub status: String,
    /// 返回码，必须符合返回码规则
    pub retcode: u32,
    /// 响应数据
    pub data: Value,
    /// 错误信息，当动作执行失败时，建议在此填写人类可读的错误信息，当执行成功时，应为空字符串
    pub message: String,
}

impl<T> From<T> for Resp
where
    T: Into<Value>,
{
    fn from(data: T) -> Self {
        Self {
            status: "ok".to_string(),
            retcode: 0,
            data: data.into(),
            message: "".to_string(),
        }
    }
}

impl<T, S> From<(T, S)> for Resp
where
    T: Into<Value>,
    S: Display,
{
    fn from(data: (T, S)) -> Self {
        Self {
            status: "ok".to_string(),
            retcode: 0,
            data: data.0.into(),
            message: data.1.to_string(),
        }
    }
}

/// failed ActionResp 构造
#[derive(Clone)]
pub struct RespError {
    pub retcode: u32,
    pub message: String,
}

impl std::fmt::Debug for RespError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RespError[{}]: {}", self.retcode, self.message)
    }
}

impl From<RespError> for Resp {
    fn from(error: RespError) -> Self {
        Self {
            status: "failed".to_string(),
            retcode: error.retcode,
            data: Value::Null,
            message: error.message,
        }
    }
}

impl<T> From<(RespError, T)> for Resp
where
    T: Into<Value>,
{
    fn from(error: (RespError, T)) -> Self {
        Self {
            status: "failed".to_string(),
            retcode: error.0.retcode,
            data: error.1.into(),
            message: error.0.message,
        }
    }
}

impl Resp {
    /// map ActionResp to `Result<Value, RespError>`
    pub fn as_result(self) -> Result<Value, RespError> {
        if self.retcode != 0 {
            Err(RespError {
                retcode: self.retcode,
                message: self.message,
            })
        } else {
            Ok(self.data)
        }
    }

    /// map and downcast ActionResp
    pub fn as_result_downcast<T>(self) -> WalleResult<T>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        match self.as_result() {
            Ok(v) => v.try_into(),
            Err(e) => Err(WalleError::RespError(e)),
        }
    }

    /// build a success RespAction
    pub fn ok<T, S>(data: T, message: S) -> Self
    where
        T: Into<Value>,
        S: Display,
    {
        Self {
            status: "ok".to_string(),
            retcode: 0,
            data: data.into(),
            message: message.to_string(),
        }
    }

    /// build a failed RespAction
    pub fn failed<T, S>(retcode: u32, data: T, message: S) -> Self
    where
        T: Into<Value>,
        S: Display,
    {
        Self {
            status: "failed".to_string(),
            retcode,
            data: data.into(),
            message: message.to_string(),
        }
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
    /// pub fn bad_request<T: std::fmt::Display>(msg: T) -> RespError {
    ///     RespError {
    ///         code: 10001,
    ///         message: {
    ///             let mut message = String::from($message);
    ///             let msg = msg.to_string();
    ///                 if msg != String::default() {
    ///                     message.push(':');
    ///                     message.push_str(&msg);
    ///                 }
    ///             message
    ///         },
    ///     }
    /// }
    /// ```
    #[macro_export]
    macro_rules! error_type {
        ($name: ident, $retcode: expr, $message: expr) => {
            pub fn $name<T: std::fmt::Display>(msg: T) -> RespError {
                RespError {
                    retcode: $retcode,
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
    error_type!(who_am_i, 10101, "Who Am I");

    error_type!(bad_handler, 20001, "动作处理器实现错误");
    error_type!(internal_handler, 20002, "动作处理器运行时抛出异常");

    error_type!(database_error, 31000, "数据库错误");
    error_type!(filesystem_error, 32000, "文件系统错误");
    error_type!(network_error, 33000, "网络错误");
    error_type!(platform_error, 34000, "机器人平台错误");
    error_type!(tired, 36000, "I Am Tired!");
}
