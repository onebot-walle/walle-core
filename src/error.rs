use thiserror::Error;

use crate::resp::RespError;

pub type WalleResult<T> = Result<T, WalleError>;

/// Walle-core errors
#[derive(Error, Debug)]
pub enum WalleError {
    // event
    #[error("expect {0} found {1}")]
    EventDeclareNotMatch(&'static str, String),
    // action
    #[error("Action send error")]
    ActionSendError,
    // resp
    #[error("Action Response Timeout")]
    ResponseTimeout,
    #[error("RespMissmatch")]
    RespNotMatch, //todo
    #[error("{0:?}")]
    RespError(RespError),
    // server
    #[error("{0}")]
    IO(#[from] std::io::Error),
    // Running Time Error
    #[error("OneBot is already running")]
    AlreadyRunning,
    #[error("OneBot is not running")]
    NotRunning,

    // Extended
    #[error("ExtendedMap missed key: {0}")]
    MapMissedKey(String),
    #[error("Type mismatch expect {0}, got {1}")]
    ValueTypeNotMatch(String, String),
    #[error("Illegal base64")]
    IllegalBase64(String),

    // OBC
    #[error("Bot not exist")]
    BotNotExist,

    #[error("{0}")]
    Other(String),
}
