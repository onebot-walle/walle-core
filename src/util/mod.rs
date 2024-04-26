//! 通用内容

use std::fmt::Debug;
use serde::{Deserialize, Serialize};

mod bytes;
mod echo;
pub mod value;

pub use bytes::*;
pub use echo::*;
pub use value::*;

/// 返回纳秒单位时间戳
pub fn timestamp_nano() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

/// 返回 f64 时间戳，单位为秒
pub fn timestamp_nano_f64() -> f64 {
    timestamp_nano() as f64 / 1_000_000_000.0
}

/// 从纳秒时间戳生成 uuid
#[cfg(feature = "impl-obc")]
pub fn new_uuid() -> String {
    uuid::Uuid::from_u128(timestamp_nano()).to_string()
}

/// 约束具有 `self` 字段
pub trait GetSelf: Sized {
    fn get_self(&self) -> Selft;
}

#[doc(hidden)]
pub trait ProtocolItem:
    Serialize + for<'de> Deserialize<'de> + Debug + Send + Sync + 'static
{
    fn json_encode(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn json_decode(s: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
    fn rmp_encode(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }
    fn rmp_decode(v: &[u8]) -> Result<Self, String>
    where
        Self: Sized,
    {
        rmp_serde::from_slice(v).map_err(|e| e.to_string())
    }
    #[cfg(feature = "http")]
    fn to_body(self, content_type: &ContentType) -> hyper::Body {
        match content_type {
            ContentType::Json => hyper::Body::from(self.json_encode()),
            ContentType::MsgPack => hyper::Body::from(self.rmp_encode()),
        }
    }
    #[cfg(feature = "websocket")]
    fn to_ws_msg(self, content_type: &ContentType) -> tokio_tungstenite::tungstenite::Message {
        match content_type {
            ContentType::Json => tokio_tungstenite::tungstenite::Message::Text(self.json_encode()),
            ContentType::MsgPack => {
                tokio_tungstenite::tungstenite::Message::Binary(self.rmp_encode())
            }
        }
    }
}

impl<T> ProtocolItem for T where
    T: Serialize + for<'de> Deserialize<'de> + Debug + Send + Sync + 'static
{
}

/// Onebot 协议支持的数据编码格式
///
/// Json or MessagePack
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Json,
    MsgPack,
}

impl ContentType {
    #[allow(dead_code)]
    pub fn new(s: &str) -> Option<Self> {
        match s {
            "application/json" => Some(Self::Json),
            "application/msgpack" => Some(Self::MsgPack),
            _ => None,
        }
    }
}

impl std::fmt::Display for ContentType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Json => write!(f, "application/json"),
            Self::MsgPack => write!(f, "application/msgpack"),
        }
    }
}

#[doc(hidden)]
pub(crate) trait AuthReqHeaderExt {
    fn header_auth_token(self, token: &Option<String>) -> Self;
}

#[cfg(feature = "http")]
use hyper::http::{header::AUTHORIZATION, request::Builder};
#[cfg(all(feature = "websocket", not(feature = "http")))]
use tokio_tungstenite::tungstenite::http::{header::AUTHORIZATION, request::Builder};

use crate::{structs::Selft, WalleResult};

#[cfg(any(feature = "websocket", feature = "http"))]
impl AuthReqHeaderExt for Builder {
    fn header_auth_token(self, token: &Option<String>) -> Self {
        if let Some(token) = token {
            self.header(AUTHORIZATION, format!("Bearer {}", token))
        } else {
            self
        }
    }
}

#[doc(hidden)]
pub trait TryAsRef<'a, T>
where
    T: 'a,
{
    fn _try_as_ref(&'a self) -> WalleResult<T>;
}

#[doc(hidden)]
pub trait TryAsMut<'a, T>
where
    T: 'a,
{
    fn _try_as_mut(&'a mut self) -> WalleResult<T>;
}
