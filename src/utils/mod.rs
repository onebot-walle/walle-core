mod map;
pub use map::*;
mod alt;
pub use alt::*;

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
pub(crate) struct Echo<I> {
    #[serde(flatten)]
    pub inner: I,
    pub echo: Option<EchoInner>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) struct EchoS(Option<EchoInner>);

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub(crate) enum EchoInner {
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
    pub(crate) fn unpack(self) -> (I, EchoS) {
        (self.inner, EchoS(self.echo))
    }
}

impl EchoS {
    pub(crate) fn pack<I>(&self, i: I) -> Echo<I> {
        Echo {
            inner: i,
            echo: self.0.clone(),
        }
    }

    pub(crate) fn new(tag: &str) -> Self {
        return Self(Some(EchoInner::S(format!("{}-{}", tag, timestamp_nano()))));
    }
}

/// Event 模型 self_id 字段约束
pub trait SelfId: Sized {
    fn self_id(&self) -> String;
}

#[doc(hidden)]
pub trait ProtocolItem: Serialize + for<'de> Deserialize<'de> {
    fn json_encode(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn json_decode(s: &str) -> Result<Self, String>
    where
        Self: Sized,
    {
        serde_json::from_str(s).map_err(|e| e.to_string())
    }
    fn json_from_reader<R: std::io::Read>(r: R) -> Result<Self, String>
    where
        Self: Sized,
    {
        serde_json::from_reader(r).map_err(|e| e.to_string())
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
    fn rmp_from_reader<R: std::io::Read>(r: R) -> Result<Self, String>
    where
        Self: Sized,
    {
        rmp_serde::from_read(r).map_err(|e| e.to_string())
    }
}

impl<T> ProtocolItem for T where T: Serialize + for<'de> Deserialize<'de> {}

/// Onebot 协议支持的数据编码格式
///
/// Json or MessagePack
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