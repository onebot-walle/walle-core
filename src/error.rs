use thiserror::Error;

pub type WalleResult<T> = Result<T, WalleError>;

#[derive(Error, Debug)]
pub enum WalleError {
    /// action
    #[error("Action send error")]
    ActionSendError,
    /// resp
    #[error("Action Response Timeout")]
    ResponseTimeout,
    /// server
    #[error("{0}")]
    IO(#[from] std::io::Error),
    /// Running Time Error
    #[error("OneBot is already running")]
    AlreadyRunning,

    // ExtendedMap
    #[error("ExtendedMap missed key: {0}")]
    MapMissedKey(String),
    #[error("ExtendedMap value type mismatch: expect {0}, got {1}")]
    MapValueTypeMismatch(String, String),

    /// Resp
    #[error("RespMissmatch")]
    RespMissmatch, //todo
}
