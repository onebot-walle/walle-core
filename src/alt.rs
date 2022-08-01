use std::sync::Arc;

use async_trait::async_trait;
use colored::*;
use tracing::info;

use crate::action::Action;
use crate::event::Event;
use crate::prelude::WalleResult;
use crate::resp::{resp_error, RespError};
use crate::util::{SelfIds, Value, ValueMap};
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
        format!("[{}] {}", h.bright_blue(), &self.extra.colored_alt())
    }
}

impl ColoredAlt for Action {
    fn colored_alt(&self) -> String {
        format!(
            "[{}] {}",
            self.action.bright_yellow(),
            &self.params.colored_alt()
        )
    }
}

impl ColoredAlt for ValueMap {
    fn colored_alt(&self) -> String {
        self.iter()
            .map(|(k, v)| {
                if k == "message" && v.is_list() {
                    use crate::segment::{alt, Segments};
                    if let Ok(segs) = Segments::try_from(v.clone()) {
                        return format!("{}: \"{}\"", k, alt(&segs));
                    }
                }
                format!("{}: {}", k, v.colored_alt())
            })
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl ColoredAlt for Vec<Value> {
    fn colored_alt(&self) -> String {
        self.iter()
            .map(|v| v.colored_alt())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

impl ColoredAlt for Value {
    fn colored_alt(&self) -> String {
        match self {
            Value::Bool(b) => b.to_string(),
            Value::Bytes(_) => "<bytes>".to_string(),
            Value::Str(s) => format!("\"{}\"", s),
            Value::Int(i) => i.to_string(),
            Value::F64(f) => f.to_string(),
            Value::Map(m) => m.colored_alt(),
            Value::List(l) => l.colored_alt(),
            Value::Null => "null".to_string(),
        }
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
impl<E, A, R> ActionHandler<E, A, R> for TracingHandler<E, A, R>
where
    E: Send + Sync + 'static,
    A: ColoredAlt + Send + Sync + 'static,
    R: From<RespError> + Send + Sync + 'static,
{
    type Config = ();
    async fn start<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH>>,
        _config: (),
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
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
impl<E, A, R> EventHandler<E, A, R> for TracingHandler<E, A, R>
where
    E: ColoredAlt + Send + Sync + 'static,
    A: Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    type Config = ();
    async fn start<AH, EH>(
        &self,
        _ob: &Arc<OneBot<AH, EH>>,
        _config: Self::Config,
    ) -> WalleResult<Vec<tokio::task::JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
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
