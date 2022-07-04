use std::sync::Arc;

use colored::*;
use tracing::info;

use crate::action::StandardAction;
use crate::event::{
    BaseEvent, EventContent, EventDetailType, MessageContent, MessageEventDetail, NoticeContent,
    RequestContent,
};
use crate::message::MessageAlt;
use crate::prelude::WalleResult;
use crate::resp::{resp_error, RespError};
use crate::{ActionHandler, EventHandler, OneBot, WALLE_CORE};

/// 命令行着色输出，可以用于 log
pub trait ColoredAlt {
    fn colored_alt(&self) -> Option<String>;
}

impl<T: ColoredAlt> ColoredAlt for BaseEvent<T> {
    fn colored_alt(&self) -> Option<String> {
        self.content.colored_alt()
    }
}

impl ColoredAlt for EventContent {
    fn colored_alt(&self) -> Option<String> {
        match self {
            EventContent::Message(m) => m.colored_alt(),
            EventContent::Notice(n) => n.colored_alt(),
            EventContent::Request(r) => r.colored_alt(),
            _ => None,
        }
    }
}

impl ColoredAlt for MessageContent<MessageEventDetail> {
    fn colored_alt(&self) -> Option<String> {
        match &self.detail {
            MessageEventDetail::Channel {
                guild_id,
                channel_id,
                ..
            } => Some(format!(
                "[{}:{}] {} from {}",
                guild_id.blue(),
                channel_id.bright_blue(),
                self.alt_message,
                self.user_id.bright_green()
            )),
            MessageEventDetail::Group { group_id, .. } => Some(format!(
                "[{}] {} from {}",
                group_id.bright_blue(),
                self.alt_message,
                self.user_id.bright_green()
            )),
            MessageEventDetail::Private { .. } => Some(format!(
                "[{}] {}",
                self.user_id.bright_green(),
                self.alt_message
            )),
        }
    }
}

impl ColoredAlt for NoticeContent {
    fn colored_alt(&self) -> Option<String> {
        let head = format!("[{}]", self.detail_type().bright_red());
        let body = match self {
            Self::GroupMemberIncrease {
                sub_type,
                group_id,
                user_id,
                operator_id,
                ..
            } => match sub_type.as_str() {
                "invite" => format!(
                    "{} invite {} to {}",
                    operator_id.bright_red(),
                    user_id.bright_green(),
                    group_id.bright_blue()
                ),
                "join" => format!("{} join {}", user_id.bright_green(), group_id.bright_blue()),
                _ => format!("{:?}", self),
            },
            Self::GroupMemberDecrease {
                sub_type,
                group_id,
                user_id,
                operator_id,
                ..
            } => match sub_type.as_str() {
                "kick" => format!(
                    "{} kick {} out of {}",
                    operator_id.bright_red(),
                    user_id.bright_green(),
                    group_id.bright_blue()
                ),
                "leave" => format!(
                    "{} leave {}",
                    user_id.bright_green(),
                    group_id.bright_blue()
                ),
                _ => format!("{:?}", self),
            },
            _ => format!("{:?}", self), //todo
        };
        return Some(format!("{} {}", head, body));
    }
}

impl ColoredAlt for RequestContent {
    fn colored_alt(&self) -> Option<String> {
        return Some(format!(
            "[{}] {:?}",
            self.detail_type().bright_yellow(),
            self
        ));
    }
}

impl ColoredAlt for StandardAction {
    fn colored_alt(&self) -> Option<String> {
        let head = format!("[{}]", self.action_type().bright_yellow());
        let body = match self {
            StandardAction::SendMessage(c) => {
                if let Some(group_id) = &c.group_id {
                    format!("{} to {}", c.message.alt(), group_id.bright_blue())
                } else if let Some(user_id) = &c.user_id {
                    format!("{} to {}", c.message.alt(), user_id.bright_green())
                } else {
                    format!("{:?}", self)
                }
            }
            a => format!("{a:?}"), //todo
        };
        Some(format!("{head} {body}"))
    }
}

#[derive(Debug)]
pub struct TracingHandler<E, A, R>(std::marker::PhantomData<(E, A, R)>);

impl<E, A, R> Default for TracingHandler<E, A, R> {
    fn default() -> Self {
        Self(std::marker::PhantomData::default())
    }
}

#[async_trait::async_trait]
impl<E, A, R, const V: u8> ActionHandler<E, A, R, V> for TracingHandler<E, A, R>
where
    E: Send + Sync + 'static,
    A: ColoredAlt + Send + Sync + 'static,
    R: From<RespError> + Send + Sync + 'static,
{
    type Config = ();
    async fn start<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH, V>>,
        _config: (),
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static,
    {
        Ok(vec![])
    }
    async fn call<AH, EH>(&self, action: A, _ob: &OneBot<AH, EH, V>) -> WalleResult<R>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static,
    {
        if let Some(alt) = action.colored_alt() {
            info!(target: WALLE_CORE, alt);
        }
        Ok(resp_error::unsupported_action("").into())
    }
}

#[async_trait::async_trait]
impl<E, A, R, const V: u8> EventHandler<E, A, R, V> for TracingHandler<E, A, R>
where
    E: ColoredAlt + Send + Sync + 'static,
    A: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    type Config = ();
    async fn start<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH, V>>,
        _config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static,
    {
        Ok(vec![])
    }
    async fn call<AH, EH>(&self, event: E, _ob: &OneBot<AH, EH, V>)
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static,
    {
        if let Some(alt) = event.colored_alt() {
            info!(target: WALLE_CORE, alt);
        }
    }
}
