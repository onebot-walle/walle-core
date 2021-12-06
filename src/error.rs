use thiserror::Error;
use tokio_tungstenite::tungstenite::Error as WsError;

pub type WalleResult<T> = std::result::Result<T, WalleError>;

#[derive(Error, Debug)]
pub enum WalleError {
    // serde
    #[error("Serde Json error: {0}")]
    SerdeJsonError(serde_json::Error),
    // comms
    #[error("Authorization failed")]
    AuthorizationFailed,
    #[error("Tcpconnect connect to {0} failed")]
    TcpConnectFailed(std::io::Error),
    #[error("Websocket upgrade failed: {0}")]
    WebsocketUpgradeFail(WsError),
    #[error("Websocket link has no peer address")]
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

pub(crate) trait WalleLogExt: Sized {
    fn log_err(&self) {}
    fn info(self, s: String) -> Self {
        tracing::info!(target: "Walle-core", "{}", s);
        self
    }
    fn debug(self, s: String) -> Self {
        tracing::debug!(target: "Walle-core", "{}", s);
        self
    }
    fn trace(self, s: String) -> Self {
        tracing::trace!(target: "Walle-core", "{}", s);
        self
    }
}

impl<T> WalleLogExt for WalleResult<T> {
    fn log_err(&self) {
        use tracing::warn;
        if let Err(e) = self {
            warn!(target: "Walle-core","{}", e);
        }
    }
}

impl WalleLogExt for WalleError {
    fn log_err(&self) {
        use tracing::warn;
        warn!(target: "Walle-core","{}", self);
    }
}
