use std::time::Duration;

use crate::{
    action::{Action, Resp},
    WalleError, WalleResult,
};

impl super::Bot {
    pub async fn call_action(&self, action: Action) -> WalleResult<Resp> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.echo_map
            .write()
            .await
            .insert(action.echo.clone(), resp_tx);
        self.action_sender
            .send(action.clone())
            .map_err(|_| WalleError::Disconnectted)?;
        tokio::time::timeout(Duration::from_secs(10), resp_rx)
            .await
            .map_err(|_| WalleError::RespTimeOut)?
            .map_err(|_| WalleError::NoResp(action.echo))
    }
}
