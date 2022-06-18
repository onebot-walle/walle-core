mod value;
use std::fmt::Debug;

pub use value::*;
mod alt;
pub use alt::*;

use crate::action::ActionType;

pub fn timestamp_nano() -> u128 {
    use std::time;

    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

#[cfg(all(feature = "websocket", feature = "impl"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "websocket", feature = "impl"))))]
pub fn timestamp_nano_f64() -> f64 {
    timestamp_nano() as f64 / 1_000_000_000.0
}

#[cfg(feature = "impl")]
pub fn new_uuid() -> String {
    uuid::Uuid::from_u128(timestamp_nano()).to_string()
}

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Echo<I> {
    #[serde(flatten)]
    pub inner: I,
    pub echo: Option<EchoInner>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, Default)]
pub struct EchoS(pub Option<EchoInner>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum EchoInner {
    S(String),
    Map(String),
}

impl Serialize for EchoInner {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            EchoInner::S(s) => s.serialize(serializer),
            EchoInner::Map(s) => {
                let map: ExtendedMap = serde_json::from_str(s).unwrap();
                map.serialize(serializer)
            }
        }
    }
}

struct EchoInnerVisitor;

impl<'de> Visitor<'de> for EchoInnerVisitor {
    type Value = EchoInner;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("string or map")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(EchoInner::S(v.to_owned()))
    }

    fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
    where
        M: serde::de::MapAccess<'de>,
    {
        let mut s = ExtendedMap::new();
        while let Some(key) = map.next_key::<String>()? {
            s.insert(key, map.next_value()?);
        }
        Ok(EchoInner::Map(serde_json::to_string(&s).unwrap()))
    }
}

impl<'de> Deserialize<'de> for EchoInner {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(EchoInnerVisitor)
    }
}

impl<I> Echo<I> {
    pub fn unpack(self) -> (I, EchoS) {
        (self.inner, EchoS(self.echo))
    }

    pub fn get_echo(&self) -> EchoS {
        EchoS(self.echo.clone())
    }
}

impl<I> ActionType for Echo<I>
where
    I: ActionType,
{
    fn content_type(&self) -> crate::utils::ContentType {
        self.inner.content_type()
    }
}

impl EchoS {
    pub fn pack<I>(&self, i: I) -> Echo<I> {
        Echo {
            inner: i,
            echo: self.0.clone(),
        }
    }

    pub fn new(tag: &str) -> Self {
        Self(Some(EchoInner::S(format!("{}-{}", tag, timestamp_nano()))))
    }
}

/// Event 模型 self_id 字段约束
pub trait SelfId: Sized {
    fn self_id(&self) -> String;
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
    fn to_body(self) -> hyper::Body
    where
        Self: ActionType,
    {
        match self.content_type() {
            ContentType::Json => hyper::Body::from(self.json_encode()),
            ContentType::MsgPack => hyper::Body::from(self.rmp_encode()),
        }
    }
    #[cfg(feature = "websocket")]
    fn to_ws_msg(self) -> tokio_tungstenite::tungstenite::Message
    where
        Self: ActionType,
    {
        match self.content_type() {
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

/// 定义 extra strcut
///
/// ```rust
/// extra_struct!(DeleteMessage, message_id: String);
/// ```
/// generate code:
/// ```rust
/// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
/// pub struct DeleteMessage {
///     pub message_id: String,
///     #[serde(flatten)]
///     pub extra: ExtendedMap,
/// }
/// ```
#[macro_export]
macro_rules! extra_struct {
    ($action_name: ident, $($field_name: ident: $field_ty: ty),*) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct $action_name {
            $(pub $field_name: $field_ty,)*
            #[serde(flatten)]
            pub extra: ExtendedMap,
        }
    };
}
