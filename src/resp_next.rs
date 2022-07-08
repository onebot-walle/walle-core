use serde::{Deserialize, Serialize};

use crate::{prelude::WalleError, util::ExtendedValue};

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

pub struct RespValue<T>(pub T);

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

impl<T> TryFrom<Resp> for RespValue<T>
where
    T: TryFrom<ExtendedValue, Error = WalleError>,
{
    type Error = WalleError;
    fn try_from(value: Resp) -> Result<Self, Self::Error> {
        Ok(Self(T::try_from(value.data)?))
    }
}
