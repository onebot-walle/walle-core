use std::sync::Arc;

use async_trait::async_trait;
use core::future::Future;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

use crate::{WalleError, WalleResult};

pub mod obc;

pub trait Static: Sync + Send + 'static {}
impl<T: Sync + Send + 'static> Static for T {}

/// ECAH: EventConstructor + ActionHandler
/// EHAC: EventHandler + ActionConstructor
pub struct OneBot<AH, EH, const V: u8> {
    pub action_handler: AH,
    pub event_handler: EH,

    // Some for running, None for stopped
    signal: std::sync::Mutex<Option<broadcast::Sender<()>>>,
}

#[async_trait]
pub trait ActionHandler<E, A, R, EH, const V: u8>: Sized {
    type Config;
    async fn ecah_start(
        &self,
        ob: &Arc<OneBot<Self, EH, V>>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>;
    async fn handle_action(&self, action: A, ob: &OneBot<Self, EH, V>) -> WalleResult<R>;
}

#[async_trait]
pub trait EventHandler<E, A, R, AH, const V: u8>: Sized {
    type Config;
    async fn ehac_start(
        &self,
        ob: &Arc<OneBot<AH, Self, V>>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>;
    async fn handle_event(&self, event: E, ob: &OneBot<AH, Self, V>);
}

pub type ImplOneBot<E, AH, const V: u8> = OneBot<AH, obc::ImplOBC<E>, V>;
pub type AppOneBot<A, R, EH, const V: u8> = OneBot<obc::AppOBC<A, R>, EH, V>;

impl<AH, EH, const V: u8> OneBot<AH, EH, V> {
    pub async fn start<E, A, R>(
        self: &Arc<Self>,
        ah_config: AH::Config,
        eh_config: EH::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        E: Static,
        A: Static,
        R: Static,
        AH: ActionHandler<E, A, R, EH, V> + Static,
        EH: EventHandler<E, A, R, AH, V> + Static,
    {
        let mut signal = self.signal.lock().unwrap();
        if signal.is_none() {
            let (tx, _) = tokio::sync::broadcast::channel(1);
            *signal = Some(tx);
        } else {
            return Err(WalleError::AlreadyRunning);
        }
        drop(signal);
        let mut tasks = vec![];
        tasks.extend(
            self.action_handler
                .ecah_start(self, ah_config)
                .await?
                .into_iter(),
        );
        tasks.extend(
            self.event_handler
                .ehac_start(self, eh_config)
                .await?
                .into_iter(),
        );
        Ok(tasks)
    }
    pub fn handle_event<'a, E, A, R>(&'a self, event: E) -> impl Future<Output = ()> + 'a
    where
        EH: EventHandler<E, A, R, AH, V> + Static,
    {
        self.event_handler.handle_event(event, self)
    }
    pub fn handle_action<'a, E, A, R>(
        &'a self,
        action: A,
    ) -> impl Future<Output = WalleResult<R>> + 'a
    where
        R: Static,
        AH: ActionHandler<E, A, R, EH, V> + Static,
    {
        self.action_handler.handle_action(action, self)
    }
    pub fn shutdown(&self) -> WalleResult<()> {
        let tx = self
            .signal
            .lock()
            .unwrap()
            .take()
            .ok_or(WalleError::NotRunning)?;
        tx.send(()).ok();
        Ok(())
    }
    pub fn get_signal_rx(&self) -> WalleResult<tokio::sync::broadcast::Receiver<()>> {
        Ok(self
            .signal
            .lock()
            .unwrap()
            .as_ref()
            .ok_or(WalleError::NotRunning)?
            .subscribe())
    }
}
