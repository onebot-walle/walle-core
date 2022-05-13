use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};
use std::{fmt::Debug, sync::Arc};

#[cfg(feature = "app")]
use crate::app::ArcBot;
#[cfg(feature = "impl")]
use crate::Resps;

mod fnt;

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
pub trait ActionHandler<A, R, OB>
where
    A: DeserializeOwned + Debug + Send + 'static,
    R: Serialize + Debug + Send + 'static,
{
    async fn handle(&self, action: A, ob: &OB) -> R;
}

/// 应用端处理 Event 需要实现的 Trait
///
/// 请注意，该出的泛型 E 应为使用 CustomEvent 包装后的 Event 并非 Content
#[cfg(feature = "app")]
#[async_trait]
pub trait EventHandler<E, A, R>
where
    E: Clone + DeserializeOwned + Send + 'static,
{
    async fn handle(&self, bot: ArcBot<A, R>, event: E);
}

/// 内置默认 Handler ，可以使用 `DefaultHandler::arc()` 返回打包后的 Handler 直接使用  ( 两种 Handler trait 均已实现 )
///
/// 仅使用默认 Onebot 类型 (Event, Action, ActionResps) 时可用
///
/// # 默认实现
///
/// - ActionHandler: 返回默认 Version 信息，其余均返回 unsupported
/// - EventHandler: 仅 Log 打印输出 Event
pub struct DefaultHandler;

impl DefaultHandler {
    pub fn arc() -> Arc<Self> {
        Arc::new(Self)
    }
}

#[cfg(feature = "impl")]
#[async_trait]
impl<E, const V: u8>
    ActionHandler<
        crate::StandardAction,
        crate::Resps,
        crate::impls::CustomOneBot<E, crate::StandardAction, crate::Resps, V>,
    > for DefaultHandler
where
    E: Send,
{
    async fn handle(
        &self,
        action: crate::StandardAction,
        _ob: &crate::impls::CustomOneBot<E, crate::StandardAction, crate::Resps, V>,
    ) -> Resps {
        use crate::{
            resp::{Resp, RespContent},
            StandardAction,
        };

        match action {
            StandardAction::GetVersion(_) => {
                Resp::success(RespContent::Version(get_version().await))
            }
            _ => Resp::unsupported_action(),
        }
    }
}

#[cfg(feature = "app")]
#[async_trait]
impl<A, R> EventHandler<crate::StandardEvent, A, R> for DefaultHandler {
    async fn handle(&self, _: ArcBot<A, R>, event: crate::StandardEvent) {
        use crate::EventContent;
        use colored::*;
        use tracing::{debug, info, trace};

        match &event.content {
            EventContent::Meta(m) => debug!(
                target: "Walle-core",
                "[{}] MetaEvent -> Type {}",
                event.self_id.red(),
                m.detail_type().green()
            ),
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
                    target: "Walle-core",
                    "[{}] MessageEvent -> from {} alt {}",
                    event.self_id.red(),
                    m.user_id.blue(),
                    alt.green()
                )
            }
            EventContent::Notice(_) => {
                trace!(target: "Walle-core","[{}] NoticeEvent ->", event.self_id.red())
            }
            EventContent::Request(_) => {
                info!(target: "Walle-core","[{}] RequestEvent ->", event.self_id.red())
            }
        }
    }
}

#[cfg(feature = "impl")]
async fn get_version() -> crate::resp::VersionContent {
    crate::resp::VersionContent::default()
}
