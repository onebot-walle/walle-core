pub fn timestamp() -> i64 {
    use std::time;

    time::SystemTime::now()
        .duration_since(time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

#[cfg(feature = "echo")]
use serde::{Deserialize, Serialize};
#[cfg(feature = "echo")]
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Echo<I> {
    #[serde(flatten)]
    pub inner: I,
    pub echo: String,
}
