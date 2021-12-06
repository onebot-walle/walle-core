use thiserror::Error;

pub type WalleResult<T> = std::result::Result<T, WalleError>;

#[derive(Error, Debug)]
pub enum WalleError {
    // comms
    #[error("Authorization failed")]
    AuthorizationFailed,
    #[error("Tcpconnect failed")]
    TcpConnectFailed,
    #[error("Websocket upgrade failed")]
    WebsocketUpgradeFail,
    #[error("Websocket link has no address")]
    WebsocketNoAddress,
    /// action
    #[error("Action send error")]
    ActionSendError,
    /// resp
    #[error("Action Response Timeout")]
    ActionResponseTimeout,
    #[error("Action Response RecvError:{0}")]
    ActionResponseRecvError(tokio::sync::oneshot::error::RecvError),
    /// server
    #[error("TcpServer bind address error: {0}")]
    TcpServerBindAddressError(std::io::Error),
    /// Running Time Error
    #[error("OneBot is already running")]
    AlreadyRunning,
}
