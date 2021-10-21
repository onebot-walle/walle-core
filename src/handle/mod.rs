use async_trait::async_trait;

#[async_trait]
pub trait ActionHandler {
    async fn handle(&self, action: crate::Action) -> Option<crate::ActionResps>;
}

pub struct DefaultHandler;

#[async_trait]
impl ActionHandler for DefaultHandler {
    async fn handle(&self, action: crate::Action) -> Option<crate::ActionResps> {
        use crate::{
            action_resp::{ActionResp, ActionRespContent},
            Action,
        };

        match action {
            Action::GetVersion => Some(ActionResp::success(ActionRespContent::Version(
                get_version().await,
            ))),
            _ => None,
        }
    }
}

async fn get_version() -> crate::action_resp::VersionContent {
    crate::action_resp::VersionContent::default()
}

