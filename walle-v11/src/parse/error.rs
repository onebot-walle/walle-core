use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalleParseError {
    #[error("MessageSegment {0} missed field {1}")]
    MsgSegMissedField(String, String),
    #[error("MessageSegment {0} field {1} type mismatch: expect {2}, got {3}")]
    MsgSegFieldTypeMismatch(String, String, String, String),

    #[error("Parse id failed: {0}")]
    Id(ParseIntError),
    #[error("{0} not support yet")]
    Todo(&'static str),
    #[error("{0}")]
    Other(String),
}
