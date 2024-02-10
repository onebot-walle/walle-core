use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::string::String as ArcStr;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Request {
    // 请求 API 端点
    action: ArcStr,
    // 请求参数
    params: Value,
    // 回显字段
    echo: ArcStr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Response {
    // 状态，ok 为成功，其他将在下文中详细说明
    status: ArcStr,
    // 返回码，0 为成功，非 0 为失败
    retcode: u32,
    // 错误信息，仅在 API 调用失败时出现
    msg: ArcStr,
    // 对错误信息的描述，仅在 API 调用失败时出现
    wording: ArcStr,
    data: HashMap<ArcStr, Value>,
    // 回显字段
    echo: ArcStr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct V11Event {
    // 时间戳
    pub time: i64,
    // 机器人QQ
    pub self_id: i64,
    // 上报数据
    #[serde(flatten)]
    pub post_type: Post,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "post_type")]
pub enum Post {
    #[serde(rename = "message")]
    Message(MessageEvent),
    #[serde(rename = "message_sent")]
    MessageSent(HashMap<ArcStr, Value>),
    #[serde(rename = "notice")]
    Notice(HashMap<ArcStr, Value>),
    #[serde(rename = "request")]
    Request(HashMap<ArcStr, Value>),
    #[serde(rename = "meta_event")]
    Meta(MetaEvent),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "meta_event_type")]
pub enum MetaEvent {
    #[serde(rename = "heartbeat")]
    HeartBeatEvent {
        interval: u32,
        sub_type: ArcStr,
        status: Status,
    },
    #[serde(rename = "lifecycle")]
    LifecycleEvent { sub_type: ArcStr, status: Status },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "message_type")]
pub enum MessageEvent {
    #[serde(rename = "private")]
    PrivateMessage {
        sub_type: V11MsgSubType, // (消息子类型)[https://whitechi73.github.io/OpenShamrock/event/general-data.html#messagesubtype] normal/friend/...
        message_id: i64,         // 消息 ID
        user_id: i64,            // 发送者 QQ 号
        message: Vec<V11MsgSegment>, // 消息内容
        raw_message: ArcStr,     // CQ 码格式消息
        sender: V11Sender,
        target_id: Option<i64>,   // 消息目标（私聊）
        temp_source: Option<i32>, // 临时聊天来源（私聊）
        peer_id: i64,             // 消息接收者，群聊是群号，私聊时是目标QQ
    },
    #[serde(rename = "group")]
    GroupMessage {
        sub_type: V11MsgSubType, // (消息子类型)[https://whitechi73.github.io/OpenShamrock/event/general-data.html#messagesubtype] normal/friend/...
        message_id: i64,         // 消息 ID
        user_id: i64,            // 发送者 QQ 号
        message: Vec<V11MsgSegment>, // 消息内容
        raw_message: ArcStr,     // CQ 码格式消息
        sender: V11Sender,
        group_id: Option<i64>,    // 群号
        target_id: Option<i64>,   // 消息目标（私聊）
        temp_source: Option<i32>, // 临时聊天来源（私聊）
        peer_id: i64,             // 消息接收者，群聊是群号，私聊时是目标QQ
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// Adjacently tagged
#[serde(tag = "type", content = "data")]
pub enum V11MsgSegment {
    #[serde(rename = "text")]
    Text { text: ArcStr },
    #[serde(rename = "face")]
    Face { id: ArcStr },
    #[serde(rename = "image")]
    Image {
        /// 图片文件地址 file:// or http(s):// or base64://
        file: ArcStr,
        /// 图片类型, 分为show, flash, original
        #[serde(rename = "type")]
        ty: ArcStr,
        /// 图片链接地址
        url: ArcStr,
    },
    #[serde(rename = "at")]
    At { qq: ArcStr },
    #[serde(rename = "poke")]
    Poke {
        #[serde(rename = "type")]
        ty: i32,
        id: i32,
        name: Option<ArcStr>,
    },
    #[serde(rename = "reply")]
    Reply { id: i32 },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct V11Sender {
    /// 发送者 QQ 号
    pub user_id: u32,
    /// 发送者昵称
    pub nickname: ArcStr,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum V11MsgSubType {
    /// 好友消息
    Friend,
    /// 群消息
    Normal,
    /// 群临时消息
    Group,
    /// 群消息(自身操作)
    GroupSelf,
    /// 系统提示
    Notice,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum V11MsgType {
    /// 私聊消息
    Private,
    /// 群消息
    Group,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Status {
    pub good: bool,
    pub online: bool,
    #[serde(rename = "qq.status")]
    pub qq_status: ArcStr,
    #[serde(rename = "self")]
    pub bot_self: BotSelf,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BotSelf {
    pub platform: ArcStr,
    pub user_id: u32,
}
