use std::net::SocketAddr;

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
}
