#[cfg(feature = "impl")]
pub(crate) fn timestamp() -> u64 {
    use std::time;

    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn timestamp_nano() -> u128 {
    use std::time;

    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_nanos()
}

#[cfg(feature = "impl")]
pub(crate) fn new_uuid() -> String {
    uuid::Uuid::from_u128(timestamp_nano()).to_string()
}

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Echo<I> {
    #[serde(flatten)]
    pub inner: I,
    pub echo: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct EchoS(Option<String>);

#[allow(dead_code)]
impl<I> Echo<I> {
    pub fn unpack(self) -> (I, EchoS) {
        return (self.inner, EchoS(self.echo));
    }
}
#[allow(dead_code)]
impl EchoS {
    pub fn pack<I>(&self, i: I) -> Echo<I> {
        return Echo {
            inner: i,
            echo: self.0.clone(),
        };
    }

    pub fn new(tag: &str) -> Self {
        return Self(Some(format!("{}-{}", tag, timestamp_nano())));
    }
}

use std::collections::HashMap;

/// 扩展字段 Map
pub type ExtendedMap = HashMap<String, ExtendedValue>;

/// 扩展字段 MapValue
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum ExtendedValue {
    Str(String),
    F64(f64),
    Int(i64),
    Bool(bool),
    Map(HashMap<String, ExtendedValue>),
    List(Vec<ExtendedValue>),
    Empty(crate::EmptyContent),
}

#[allow(dead_code)]
impl ExtendedValue {
    pub fn as_str(self) -> Option<String> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_f64(self) -> Option<f64> {
        match self {
            Self::F64(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_int(self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_bool(self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_map(self) -> Option<HashMap<String, ExtendedValue>> {
        match self {
            Self::Map(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_list(self) -> Option<Vec<ExtendedValue>> {
        match self {
            Self::List(v) => Some(v),
            _ => None,
        }
    }
    pub fn is_str(&self) -> bool {
        match self {
            Self::Str(_) => true,
            _ => false,
        }
    }
    pub fn is_f64(&self) -> bool {
        match self {
            Self::F64(_) => true,
            _ => false,
        }
    }
    pub fn is_int(&self) -> bool {
        match self {
            Self::Int(_) => true,
            _ => false,
        }
    }
    pub fn is_bool(&self) -> bool {
        match self {
            Self::Bool(_) => true,
            _ => false,
        }
    }
    pub fn is_map(&self) -> bool {
        match self {
            Self::Map(_) => true,
            _ => false,
        }
    }
    pub fn is_list(&self) -> bool {
        match self {
            Self::List(_) => true,
            _ => false,
        }
    }
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Empty(_) => true,
            _ => false,
        }
    }
}

pub trait FromStandard<S> {
    fn from_standard(standard: S) -> Self;
}

#[cfg(feature = "impl")]
#[cfg_attr(docsrs, doc(cfg(feature = "impl")))]
/// 创建心跳事件
pub trait HeartbeatBuild: Sized {
    fn build_heartbeat<A, R, const V: u8>(
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
