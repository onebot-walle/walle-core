use thiserror::Error;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
use tokio_tungstenite::tungstenite::Error as WsError;

pub type WalleResult<T> = std::result::Result<T, WalleError>;

#[derive(Error, Debug)]
pub enum WalleError {
    // serde
    #[error("Serde Json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    // comms
    #[error("Authorization failed")]
    AuthorizationFailed,
    #[error("Tcpconnect connect to {0} failed")]
    TcpConnectFailed(std::io::Error),
    #[cfg(feature = "websocket")]
    #[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
    #[error("Websocket upgrade failed: {0}")]
    WebsocketUpgradeFail(#[from] WsError),
    #[error("Websocket link has no peer address")]
    WebsocketNoAddress,
    /// action
    #[error("Action send error")]
    ActionSendError,
    /// resp
    #[error("Action Response Timeout")]
    ActionResponseTimeout,
    #[error("Action Response RecvError:{0}")]
    ActionResponseRecvError(#[from] tokio::sync::oneshot::error::RecvError),
    /// server
    #[error("TcpServer bind address error: {0}")]
    TcpServerBindAddressError(#[from] std::io::Error),
    /// Running Time Error
    #[error("OneBot is already running")]
    AlreadyRunning,
}

pub(crate) trait WalleLogExt: Sized {
    fn wran_err(&self);
    fn error_err(&self);
    fn info(self, s: &str) -> Self {
        tracing::info!(target: "Walle-core", "{}", s);
        self
    }
    fn debug(self, s: &str) -> Self {
        tracing::debug!(target: "Walle-core", "{}", s);
        self
    }
    fn trace(self, s: &str) -> Self {
        tracing::trace!(target: "Walle-core", "{}", s);
        self
    }
}

impl<T> WalleLogExt for WalleResult<T> {
    fn wran_err(&self) {
        if let Err(e) = self {
            e.wran_err();
        }
    }

    fn error_err(&self) {
        if let Err(e) = self {
            e.error_err();
        }
    }
}

impl WalleLogExt for WalleError {
    fn wran_err(&self) {
        use tracing::warn;
        warn!(target: "Walle-core","{}", self);
    }

    fn error_err(&self) {
        use tracing::error;
        error!(target: "Walle-core","{}", self);
    }
}
