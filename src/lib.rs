#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[doc(hidden)]
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
/// Walle-core
pub const WALLE_CORE: &str = "Walle-core";

pub mod action;
#[cfg(feature = "alt")]
pub mod alt;
pub mod config;
pub mod error;
pub mod event;
pub mod resp;
pub mod segment;
pub mod structs;
pub mod util;

mod ah;
pub use ah::{ActionHandler, GetSelfs, GetStatus, GetVersion};
mod eh;
pub use eh::EventHandler;
use tokio::task::JoinHandle;

#[cfg(any(feature = "impl-obc", feature = "app-obc"))]
pub mod obc;
#[cfg(test)]
mod test;

pub mod prelude {
    pub use super::*;
    pub use crate::error::{WalleError, WalleResult};
    pub use crate::util::{Echo, GetSelf, OneBotBytes, Value, ValueMap, ValueMapExt};
    pub use crate::{value, value_map, value_vec};
    pub use walle_macro::{PushToValueMap, ToAction, ToEvent, ToMsgSegment};
    pub use walle_macro::{TryFromAction, TryFromEvent, TryFromMsgSegment, TryFromValue};

    pub use crate::action::{Action, BaseAction, ToAction, TryFromAction};
    pub use crate::event::{BaseEvent, Event, ToEvent, TryFromEvent};
    pub use crate::resp::{resp_error, Resp};
    pub use crate::segment::{
        IntoMessage, MessageExt, MsgSegment, Segments, ToMsgSegment, TryFromMsgSegment,
    };
    pub use crate::structs::*;
}

/// 基础抽象模型，持有 ActionHandler 与 EventHandler
pub struct OneBot<AH, EH> {
    action_handler: AH,
    event_handler: EH,
    // Some for running, None for stopped
    signal: StdMutex<Option<tokio::sync::broadcast::Sender<()>>>,
    ah_tasks: Mutex<Vec<JoinHandle<()>>>,
    eh_tasks: Mutex<Vec<JoinHandle<()>>>,
}

use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

pub use crate::error::{WalleError, WalleResult};

impl<AH, EH> OneBot<AH, EH> {
    pub fn new(action_handler: AH, event_handler: EH) -> Self {
        Self {
            action_handler,
            event_handler,
            signal: StdMutex::new(None),
            ah_tasks: Mutex::default(),
            eh_tasks: Mutex::default(),
        }
    }
    pub async fn start<E, A, R>(
        self: &Arc<Self>,
        ah_config: AH::Config,
        eh_config: EH::Config,
        ah_first: bool,
    ) -> WalleResult<()>
    where
        E: Send + Sync + 'static,
        A: Send + Sync + 'static,
        R: Send + Sync + 'static,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        if !self.set_signal() {
            return Err(WalleError::AlreadyStarted);
        }
        if ah_first {
            *self.ah_tasks.lock().await = self.action_handler.start(self, ah_config).await?;
            *self.eh_tasks.lock().await = self.event_handler.start(self, eh_config).await?;
        } else {
            *self.eh_tasks.lock().await = self.event_handler.start(self, eh_config).await?;
            *self.ah_tasks.lock().await = self.action_handler.start(self, ah_config).await?;
        }
        Ok(())
    }
    pub async fn wait_all(&self) {
        let mut tasks: Vec<JoinHandle<()>> = std::mem::take(self.ah_tasks.lock().await.as_mut());
        tasks.extend(
            std::mem::take::<Vec<JoinHandle<()>>>(self.eh_tasks.lock().await.as_mut()).into_iter(),
        );
        for task in tasks {
            task.await.ok();
        }
    }
    pub fn set_signal(&self) -> bool {
        let mut signal = self.signal.lock().unwrap();
        if signal.is_none() {
            let (tx, _) = tokio::sync::broadcast::channel(1);
            *signal = Some(tx);
            true
        } else {
            false
        }
    }
    pub fn is_started(&self) -> bool {
        self.signal.lock().unwrap().is_some()
    }
    pub fn get_signal_rx(&self) -> WalleResult<tokio::sync::broadcast::Receiver<()>> {
        Ok(self
            .signal
            .lock()
            .unwrap()
            .as_ref()
            .ok_or(WalleError::NotStarted)?
            .subscribe())
    }
    pub async fn shutdown<E, A, R>(&self, ah_first: bool) -> WalleResult<()>
    where
        E: Send + Sync + 'static,
        A: Send + Sync + 'static,
        R: Send + Sync + 'static,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        let tx = self
            .signal
            .lock()
            .unwrap()
            .take()
            .ok_or(WalleError::NotStarted)?;
        tx.send(()).ok();
        if ah_first {
            self.action_handler.shutdown().await;
            self.event_handler.shutdown().await;
        } else {
            self.event_handler.shutdown().await;
            self.action_handler.shutdown().await;
        }
        Ok(self.wait_all().await)
    }
    pub async fn handle_event<E, A, R>(self: &Arc<Self>, event: E) -> WalleResult<()>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
        E: Send + 'static,
    {
        self.event_handler
            .call(
                self.action_handler.before_call_event(event, self).await?,
                self,
            )
            .await?;
        self.action_handler.after_call_event(self).await
    }
    pub async fn handle_action<E, A, R>(self: &Arc<Self>, action: A) -> WalleResult<R>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
        A: Send + 'static,
        R: Send + 'static,
    {
        self.event_handler
            .after_call_action(
                self.action_handler
                    .call(
                        self.event_handler.before_call_action(action, self).await?,
                        self,
                    )
                    .await?,
                self,
            )
            .await
    }
}

impl<AH, EH> GetStatus for OneBot<AH, EH>
where
    AH: GetStatus + Sync,
    EH: Sync,
{
    async fn is_good(&self) -> bool {
        self.action_handler.is_good().await
    }
    async fn get_status(&self) -> structs::Status {
        self.action_handler.get_status().await
    }
}

impl<AH, EH> GetSelfs for OneBot<AH, EH>
where
    AH: GetSelfs + Sync,
    EH: Sync,
{
    async fn get_impl(&self, selft: &structs::Selft) -> String {
        self.action_handler.get_impl(selft).await
    }
    async fn get_selfs(&self) -> Vec<structs::Selft> {
        self.action_handler.get_selfs().await
    }
}
