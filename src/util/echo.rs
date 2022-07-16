use super::{timestamp_nano, ValueMap};
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
                let map: ValueMap = serde_json::from_str(s).unwrap();
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
        let mut s = ValueMap::new();
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
