use std::sync::Arc;

use async_trait::async_trait;
use colored::*;
use tracing::info;

use crate::action::Action;
use crate::event::Event;
use crate::prelude::WalleResult;
use crate::resp::{resp_error, RespError};
use crate::util::SelfIds;
use crate::{ActionHandler, EventHandler, OneBot, WALLE_CORE};

/// 命令行着色输出，可以用于 log
pub trait ColoredAlt {
    fn colored_alt(&self) -> String;
}

impl ColoredAlt for Event {
    fn colored_alt(&self) -> String {
        let h: String = [
            self.ty.clone(),
            self.detail_type.clone(),
            self.sub_type.clone(),
        ]
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join(".");
        format!(
            "[{}] {}",
            h.bright_blue(),
            serde_json::to_string(&self.extra).unwrap()
        )
    }
}

impl ColoredAlt for Action {
    fn colored_alt(&self) -> String {
        format!(
            "[{}] {}",
            self.action.bright_yellow(),
            serde_json::to_string(&self.params).unwrap()
        )
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
impl<E, A, R> SelfIds for TracingHandler<E, A, R>
where
    E: Send + Sync + 'static,
    A: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    async fn self_ids(&self) -> Vec<String> {
        vec![]
    }
}

impl<E, A, R> crate::GetStatus for TracingHandler<E, A, R> {
    fn get_status(&self) -> crate::structs::Status {
        crate::structs::Status {
            good: true,
            online: true,
        }
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
    async fn call(&self, action: A) -> WalleResult<R> {
        info!(target: WALLE_CORE, "{}", action.colored_alt());
        Ok(resp_error::unsupported_action("").into())
    }
    async fn shutdown(&self) {
        info!(target: WALLE_CORE, "Shutting down TracingHandler")
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
    async fn call(&self, event: E) -> WalleResult<()> {
        info!(target: WALLE_CORE, "{}", event.colored_alt());
        Ok(())
    }
    async fn shutdown(&self) {
        info!(target: WALLE_CORE, "Shutting down TracingHandler")
    }
}
