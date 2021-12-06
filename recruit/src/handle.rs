use serde::{Deserialize, Serialize};
use tracing::trace;
use walle_core::{
    action::ExtendedAction, action_resp::SendMessageRespContent, async_trait, Action,
    ActionHandler, ActionRespContent, ActionResps,
};

pub(crate) struct Handler;

#[async_trait]
impl ActionHandler<ExtendedAction<ExtendAction>, ActionResps> for Handler {
    async fn handle(&self, action: ExtendedAction<ExtendAction>) -> ActionResps {
        trace!("get Action: {:?}", action);
        match action {
            ExtendedAction::Extended(ea) => self.handle(ea).await,
            ExtendedAction::Standard(sa) => self.handle(sa).await,
        }
    }
}

#[async_trait]
impl ActionHandler<Action, ActionResps> for Handler {
    async fn handle(&self, action: Action) -> ActionResps {
        match action {
            Action::SendMessage(m) => {
                ActionResps::success(ActionRespContent::SendMessage(SendMessageRespContent {
                    message_id: format!("SendMessage Action {:?} is received, but not send", m),
                    time: 0,
                }))
            }
            Action::GetVersion(_) => {
                ActionResps::success(ActionRespContent::Version(crate::core::version()))
            }
            _ => ActionResps::unsupported_action(),
        }
    }
}

#[async_trait]
impl ActionHandler<ExtendAction, ActionResps> for Handler {
    async fn handle(&self, action: ExtendAction) -> ActionResps {
        match action {
            ExtendAction::Echo { message: _ } => ActionResps::success(ActionRespContent::empty()),
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
