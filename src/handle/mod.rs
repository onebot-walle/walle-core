use async_trait::async_trait;

#[async_trait]
pub trait ActionHandler {
    type Content;
    async fn handle(action: crate::Action) -> crate::ActionResp<Self::Content>;
}

#[async_trait]
pub trait ActionHanders {
    async fn handle(action: crate::Action) -> String;
}

pub struct DoNothing;

#[async_trait]
impl ActionHandler for DoNothing {
    type Content = ();
    async fn handle(_: crate::Action) -> crate::ActionResp<Self::Content> {
        crate::ActionResp::success(())
    }
}
