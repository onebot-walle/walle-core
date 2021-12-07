use std::num::ParseIntError;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalleParseError {
    #[error("Parse MessageSegment failed: {0}")]
    MessageSegment(&'static str),
    #[error("Parse id failed: {0}")]
    Id(ParseIntError),
    #[error("Todo, {0} not support yet")]
    Todo(&'static str),
}
