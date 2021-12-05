use thiserror::Error;

pub type WalleResult<T> = Result<T, WalleError>;

#[derive(Error, Debug)]
pub enum WalleError {
    #[error("action echo:{0} has no resp")]
    NoResp(String),
    #[error("action wait resp timeout")]
    RespTimeOut,
    #[error("bot is disconnectted")]
    Disconnectted,
}
