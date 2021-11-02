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

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub(crate) struct Echo<I> {
    #[serde(flatten)]
    pub inner: I,
    pub echo: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub(crate) struct EchoS(Option<String>);

impl<I> Echo<I> {
    pub fn unpack(self) -> (I, EchoS) {
        return (self.inner, EchoS(self.echo));
    }
}

impl EchoS {
    pub fn pack<I>(self, i: I) -> Echo<I> {
        return Echo {
            inner: i,
            echo: self.0,
        };
    }

    pub fn new(tag: &str) -> Self {
        return Self(Some(format!("{}-{}", tag, timestamp_nano())));
    }
}
