mod map;
pub use map::*;

pub fn timestamp_nano() -> u128 {
    use std::time;

    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

pub fn timestamp_nano_f64() -> f64 {
    timestamp_nano() as f64 / 1_000_000_000.0
}

#[cfg(feature = "impl")]
pub(crate) fn new_uuid() -> String {
    uuid::Uuid::from_u128(timestamp_nano()).to_string()
}

use serde::{de::Visitor, Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Echo<I> {
    #[serde(flatten)]
    pub inner: I,
    pub echo: Option<EchoInner>,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct EchoS(Option<EchoInner>);

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

#[allow(dead_code)]
impl<I> Echo<I> {
    pub fn unpack(self) -> (I, EchoS) {
        (self.inner, EchoS(self.echo))
    }
}
#[allow(dead_code)]
impl EchoS {
    pub fn pack<I>(&self, i: I) -> Echo<I> {
        Echo {
            inner: i,
            echo: self.0.clone(),
        }
    }

    pub fn new(tag: &str) -> Self {
        return Self(Some(EchoInner::S(format!("{}-{}", tag, timestamp_nano()))));
    }
}

pub trait FromStandard<S> {
    fn from_standard(standard: S) -> Self;
}

#[cfg(feature = "impl")]
#[cfg_attr(docsrs, doc(cfg(feature = "impl")))]
use async_trait::async_trait;

#[cfg(feature = "impl")]
#[cfg_attr(docsrs, doc(cfg(feature = "impl")))]
#[async_trait]
/// 创建心跳事件
pub trait HeartbeatBuild: Sized {
    async fn build_heartbeat<A, R, const V: u8>(
        ob: &crate::impls::CustomOneBot<Self, A, R, V>,
        interval: u32,
    ) -> Self;
}

pub trait BasicEvent: Sized {
    fn self_id(&self) -> String;
}

/// 空结构体，用于对应 Json 中的空 Map
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmptyContent {}
