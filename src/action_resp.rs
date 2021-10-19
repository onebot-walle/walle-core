use serde::{Serialize,Deserialize};

use crate::Events;

/// *动作响应*是 OneBot 实现收到应用端的动作请求并处理完毕后，发回应用端的数据。
#[derive(Serialize, Deserialize)]
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

#[cfg(feature = "echo")]
#[derive(Serialize, Deserialize)]
pub struct EchoActionResp<T> {
    #[serde(flatten)]
    pub action_resp: ActionResp<T>,
    pub echo: String,
}

pub type LatestEvents = ActionResp<Vec<Events>>;
pub type SupportActions = ActionResp<Vec<String>>;
pub type Status = ActionResp<StatusContent>;
pub type Version = ActionResp<VersionContent>;

#[derive(Serialize, Deserialize)]
pub struct StatusContent {
    pub good: bool,
    pub online: bool,
}

#[derive(Serialize, Deserialize)]
pub struct VersionContent {
    pub r#impl: String,
    pub platform: String,
    pub version: String,
    pub onebot_version: String,
}

impl Default for Version {
    fn default() -> Self {
       Self::success(VersionContent { 
            r#impl: "AbrasOneBot".to_owned(),
            platform: "RustOneBot".to_owned(),
            version: "0.0.1".to_owned(),
            onebot_version: "12".to_owned(),
        })
    }
}
