use std::sync::Arc;

use async_trait::async_trait;
use colored::*;
use tracing::info;

use crate::action::StandardAction;
use crate::event::{
    BaseEvent, EventContent, EventDetailType, MessageContent, MessageEventDetail, NoticeContent,
    RequestContent,
};
use crate::message::MessageAlt;
use crate::prelude::{EventSubType, EventType, WalleResult};
use crate::resp::{resp_error, RespError};
use crate::{ActionHandler, EventHandler, OneBot, WALLE_CORE};

/// 命令行着色输出，可以用于 log
pub trait ColoredAlt {
    fn colored_alt(&self) -> Option<String>;
}

fn type_tree(v: Vec<&str>) -> String {
    let mut out = String::default();
    for s in v {
        if !s.is_empty() {
            out.push('.');
            out.push_str(s)
        }
    }
    out.remove(0);
    out
}

impl<T: ColoredAlt + EventType> ColoredAlt for BaseEvent<T> {
    fn colored_alt(&self) -> Option<String> {
        self.content.colored_alt().map(|alt| {
            format!(
                "[{}] {}",
                type_tree(vec![self.ty(), self.detail_type(), self.sub_type()]),
                alt
            )
        })
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
                "{} from {}:{}:{}",
                self.alt_message,
                guild_id.blue(),
                channel_id.bright_blue(),
                self.user_id.bright_green()
            )),
            MessageEventDetail::Group { group_id, .. } => Some(format!(
                "{} from {}:{}",
                self.alt_message,
                group_id.bright_blue(),
                self.user_id.bright_green()
            )),
            MessageEventDetail::Private { .. } => Some(format!(
                "{} from {}",
                self.alt_message,
                self.user_id.bright_green()
            )),
        }
    }
}

impl ColoredAlt for NoticeContent {
    fn colored_alt(&self) -> Option<String> {
        Some(format!("{}", serde_json::to_string(self).unwrap()))
    }
}

impl ColoredAlt for RequestContent {
    fn colored_alt(&self) -> Option<String> {
        Some(format!("{}", serde_json::to_string(&self.extra).unwrap()))
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
            a => format!("{}", serde_json::to_string(a).unwrap()), //todo
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

#[async_trait]
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

#[async_trait]
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
