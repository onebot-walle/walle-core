use serde::{Deserialize, Serialize};

use crate::{
    prelude::{WalleError, WalleResult},
    util::ExtendedValue,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Resp {
    /// 执行状态（成功与否），必须是 ok、failed 中的一个，分别表示执行成功和失败
    pub status: String,
    /// 返回码，必须符合返回码规则
    pub retcode: u32,
    /// 响应数据
    pub data: ExtendedValue,
    /// 错误信息，当动作执行失败时，建议在此填写人类可读的错误信息，当执行成功时，应为空字符串
    pub message: String,
}

impl<T> From<T> for Resp
where
    T: Into<ExtendedValue>,
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

impl Resp {
    pub fn as_result(self) -> WalleResult<ExtendedValue> {
        if self.retcode != 0 {
            Err(WalleError::RespError(RespError {
                retcode: self.retcode,
                message: self.message,
            }))
        } else {
            Ok(self.data)
        }
    }
}
