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

pub trait SelfId: Sized {
    fn self_id(&self) -> String;
}

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

pub trait AsStandard<T> {
    fn as_standard(&self) -> &T;
}
