use serde::{Deserialize, Serialize};
use tracing::trace;
use walle_core::{
    action::ExtendedAction, resp::SendMessageRespContent, async_trait, Action,
    ActionHandler, RespContent, Resps,
};

pub(crate) struct Handler;

#[async_trait]
impl ActionHandler<ExtendedAction<ExtendAction>, Resps> for Handler {
    async fn handle(&self, action: ExtendedAction<ExtendAction>) -> Resps {
        trace!("get Action: {:?}", action);
        match action {
            ExtendedAction::Extended(ea) => self.handle(ea).await,
            ExtendedAction::Standard(sa) => self.handle(sa).await,
        }
    }
}

#[async_trait]
impl ActionHandler<Action, Resps> for Handler {
    async fn handle(&self, action: Action) -> Resps {
        match action {
            Action::SendMessage(m) => {
                Resps::success(RespContent::SendMessage(SendMessageRespContent {
                    message_id: format!("SendMessage Action {:?} is received, but not send", m),
                    time: 0,
                }))
            }
            Action::GetVersion(_) => {
                Resps::success(RespContent::Version(crate::core::version()))
            }
            _ => Resps::unsupported_action(),
        }
    }
}

#[async_trait]
impl ActionHandler<ExtendAction, Resps> for Handler {
    async fn handle(&self, action: ExtendAction) -> Resps {
        match action {
            ExtendAction::Echo { message: _ } => Resps::success(RespContent::empty()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "params")]
#[serde(rename_all = "snake_case")]
pub enum ExtendAction {
    // meta action
    Echo { message: walle_core::Message },
}
