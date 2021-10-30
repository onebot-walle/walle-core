use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::ActionResp;

/// 处理 Action 需要实现的 Trait
///
/// 请注意，请务必实现默认返回 `ActionResp::unsupported_action()`
///
/// **ToDo:** 预计未来会写一个宏来方便使用
///
/// # Example
///
/// ```rust
/// use async_trait::async_trait;
/// use abras_onebot::{Action, ActionResps, VersionContent, ActionHandler, ActionRespContent};
///
/// pub struct DefaultHandler;
///
/// #[async_trait]
/// impl ActionHandler for DefaultHandler {
///     async fn handle(&self, action: Action) -> ActionResps {
///         match action {
///             Action::GetVersion(_) => ActionResps::success(ActionRespContent::Version(
///                 get_version().await,
///             )),
///             _ => ActionResps::unsupported_action(),
///         }
///     }
/// }
///
/// async fn get_version() -> VersionContent {
///     VersionContent::default()
/// }
/// ```
#[async_trait]
pub trait ActionHandler<A, R>
where
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    async fn handle(&self, action: A) -> ActionResp<R>;
}

/// 处理 Event 需要实现的 Trait
#[async_trait]
pub trait EventHandler<E>
where
    E: Clone + DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    async fn handle(&self, event: E);
}

#[async_trait]
pub trait ActionRespHandler<R>
where
    R: Clone + DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    async fn handle(&self, resp: R);
}

pub struct DefaultHandler;

#[async_trait]
impl ActionHandler<crate::Action, crate::ActionRespContent> for DefaultHandler {
    async fn handle(&self, action: crate::Action) -> crate::ActionResps {
        use crate::{
            action_resp::{ActionResp, ActionRespContent},
            Action,
        };

        match action {
            Action::GetVersion(_) => {
                ActionResp::success(ActionRespContent::Version(get_version().await))
            }
            _ => ActionResp::unsupported_action(),
        }
    }
}

async fn get_version() -> crate::action_resp::VersionContent {
    crate::action_resp::VersionContent::default()
}
