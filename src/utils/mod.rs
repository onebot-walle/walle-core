mod map;
pub use map::*;

#[cfg(feature = "impl")]
pub fn timestamp() -> u64 {
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
        return Self(Some(format!("{}-{}", tag, timestamp_nano())));
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
