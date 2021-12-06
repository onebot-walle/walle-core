use std::time::Duration;

use crate::action::{Action, Resp};
use walle_core::{WalleError, WalleResult};

use super::{ActionSender, EchoMap};

impl super::Bot {
    pub fn new(bot_id: i32, action_sender: ActionSender, echo_map: EchoMap) -> Self {
        Self {
            self_id: bot_id,
            action_sender,
            echo_map,
        }
    }

    pub async fn call_action(&self, action: Action) -> WalleResult<Resp> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.echo_map
            .write()
            .await
            .insert(action.echo.clone(), resp_tx);
        self.action_sender
            .send(action.clone())
            .map_err(|_| WalleError::ActionSendError)?;
        tokio::time::timeout(Duration::from_secs(10), resp_rx)
            .await
            .map_err(|_| WalleError::ActionResponseTimeout)?
            .map_err(|e| WalleError::ActionResponseRecvError(e))
    }
}
