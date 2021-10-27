use abras_onebot::{
    async_trait, tracing::trace, Action, ActionHandler, ActionRespContent, ActionResps,
};

pub(crate) struct Handler;

#[async_trait]
impl ActionHandler<Action, ActionRespContent> for Handler {
    async fn handle(&self, action: Action) -> ActionResps {
        trace!("get Action: {:?}", action);
        match action {
            // Action::SendMessage(m) => ActionResps::bad_param(),
            Action::GetVersion(_) => {
                ActionResps::success(ActionRespContent::Version(crate::core::version()))
            }
            _ => ActionResps::unsupported_action(),
        }
    }
}
