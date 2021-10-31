use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;

/// 实现端处理 Action 需要实现的 Trait
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
    async fn handle(&self, action: A) -> R;
}

/// 应用端处理 Event 需要实现的 Trait
///
/// 请注意，该出的泛型 E 应为使用 CustomEvent 包装后的 Event 并非 Content
#[async_trait]
pub trait EventHandler<E>
where
    E: Clone + DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    async fn handle(&self, event: E);
}

/// 应用端处理 ActionResp 需要实现的 Trait
#[async_trait]
pub trait ActionRespHandler<R>
where
    R: Clone + DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    async fn handle(&self, resp: R);
}

/// 内置默认 Handler ，可以使用 `DefaultHandler::arc()` 返回打包后的 Handler 直接使用  ( 三种 Handler trait 均已实现 )
///
/// 仅使用默认 Onebot 类型 (Event, Action, ActionResps) 时可用
///
/// # 默认实现
///
/// - ActionHandler: 返回默认 Version 信息，其余均返回 unsupported
/// - EventHandler: 仅 Log 打印输出 Event
/// - ActionRespHandler: Do Nothing (累了，毁灭吧，赶紧的)
pub struct DefaultHandler;

impl DefaultHandler {
    pub fn arc() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[async_trait]
impl ActionHandler<crate::Action, crate::ActionResps> for DefaultHandler {
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

#[async_trait]
impl EventHandler<crate::Event> for DefaultHandler {
    async fn handle(&self, event: crate::Event) {
        use crate::EventContent;
        use colored::*;
        use tracing::info;

        match &event.content {
            EventContent::Meta(_) => info!("[{}] MetaEvent -> type ", event.self_id.red()),
            EventContent::Message(m) => {
                let alt = if m.alt_message.is_empty() {
                    let mut t = format!("{:?}", m.message);
                    if t.len() > 15 {
                        let _ = t.split_off(15);
                    }
                    t
                } else {
                    m.alt_message.clone()
                };
                info!(
                    "[{}] MessageEvent -> from {} alt {}",
                    event.self_id.red(),
                    m.user_id.blue(),
                    alt.green()
                )
            }
            EventContent::Notice(_) => info!("[{}] NoticeEvent -> ", event.self_id.red()),
            EventContent::Request(_) => info!("[{}]RequestEvent ->", event.self_id.red()),
        }
    }
}

#[async_trait]
impl ActionRespHandler<crate::ActionResps> for DefaultHandler {
    async fn handle(&self, _: crate::ActionResps) {}
}

async fn get_version() -> crate::action_resp::VersionContent {
    crate::action_resp::VersionContent::default()
}
