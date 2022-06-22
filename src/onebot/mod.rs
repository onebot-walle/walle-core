use std::sync::Arc;

use async_trait::async_trait;
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
pub trait ActionHandler<E, A, R, const V: u8> {
    type Config;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH, V>>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
    async fn call<AH, EH>(&self, action: A, ob: &OneBot<AH, EH, V>) -> WalleResult<R>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
}

#[async_trait]
pub trait EventHandler<E, A, R, const V: u8> {
    type Config;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH, V>>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
    async fn call<AH, EH>(&self, event: E, ob: &OneBot<AH, EH, V>)
    where
        AH: ActionHandler<E, A, R, V> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, V> + Send + Sync + 'static;
}

pub type ImplOneBot<E, AH, const V: u8> = OneBot<AH, obc::ImplOBC<E>, V>;
pub type AppOneBot<A, R, EH, const V: u8> = OneBot<obc::AppOBC<A, R>, EH, V>;

impl<AH, EH, const V: u8> OneBot<AH, EH, V> {
    pub fn new(action_handler: AH, event_handler: EH) -> Self {
        Self {
            action_handler,
            event_handler,
            signal: std::sync::Mutex::new(None),
        }
    }
    pub async fn start<E, A, R>(
        self: &Arc<Self>,
        ah_config: AH::Config,
        eh_config: EH::Config,
        ah_first: bool,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        E: Static,
        A: Static,
        R: Static,
        AH: ActionHandler<E, A, R, V> + Static,
        EH: EventHandler<E, A, R, V> + Static,
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
        if ah_first {
            tasks.extend(
                self.action_handler
                    .start(self, ah_config)
                    .await?
                    .into_iter(),
            );
            tasks.extend(self.event_handler.start(self, eh_config).await?.into_iter());
        } else {
            tasks.extend(self.event_handler.start(self, eh_config).await?.into_iter());
            tasks.extend(
                self.action_handler
                    .start(self, ah_config)
                    .await?
                    .into_iter(),
            );
        }
        Ok(tasks)
    }
    pub fn started(&self) -> bool {
        self.signal.lock().unwrap().is_some()
    }
    fn get_signal_rx(&self) -> WalleResult<broadcast::Receiver<()>> {
        Ok(self
            .signal
            .lock()
            .unwrap()
            .as_ref()
            .ok_or(WalleError::NotRunning)?
            .subscribe())
    }
    fn get_onebot_version(&self) -> u8 {
        V
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
}
