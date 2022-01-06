use crate::action::{Action, Resp};
use std::sync::Arc;
use std::time::Duration;
use walle_core::{WalleError, WalleResult};

use super::ActionSender;

pub struct Bot {
    pub self_id: i32,
    pub action_sender: ActionSender,
}

pub type ArcBot = Arc<Bot>;

impl Bot {
    pub fn new(bot_id: i32, action_sender: ActionSender) -> Self {
        Self {
            self_id: bot_id,
            action_sender,
        }
    }

    pub async fn call_action(&self, action: Action) -> WalleResult<Resp> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        self.action_sender
            .send((action, resp_tx))
            .map_err(|_| WalleError::ActionSendError)?;
        tokio::time::timeout(Duration::from_secs(10), resp_rx)
            .await
            .map_err(|_| WalleError::ActionResponseTimeout)?
            .map_err(|e| WalleError::ActionResponseRecvError(e))
    }
}
