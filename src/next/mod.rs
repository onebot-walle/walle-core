use std::sync::Arc;

use async_trait::async_trait;
use core::future::Future;
use tokio::sync::{broadcast, Mutex};
use tokio::task::JoinHandle;

use crate::{WalleError, WalleResult};

pub mod obc;

pub trait Static: Sync + Send + 'static {}
impl<T: Sync + Send + 'static> Static for T {}

/// ECAH: EventConstructor + ActionHandler
/// EHAC: EventHandler + ActionConstructor
pub struct OneBot<ECAH, EHAC, const V: u8> {
    pub ecah: ECAH,
    pub ehac: EHAC,

    // Some for running, None for stopped
    signal: Mutex<Option<broadcast::Sender<()>>>,
}

#[async_trait]
pub trait ECAHtrait<E, A, R, EHAC, const V: u8>: Sized {
    type Config;
    async fn ecah_start(
        &self,
        ob: &Arc<OneBot<Self, EHAC, V>>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>;
    async fn handle_action(&self, action: A, ob: &OneBot<Self, EHAC, V>) -> WalleResult<R>;
}

#[async_trait]
pub trait EHACtrait<E, A, R, ECAH, const V: u8>: Sized {
    type Config;
    async fn ehac_start(
        &self,
        ob: &Arc<OneBot<ECAH, Self, V>>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>;
    async fn handle_event(&self, event: E, ob: &OneBot<ECAH, Self, V>);
}

pub type ImplOneBot<E, ECAH, const V: u8> = OneBot<ECAH, obc::ImplOBC<E>, V>;
pub type AppOneBot<A, R, EHAC, const V: u8> = OneBot<obc::AppOBC<A, R>, EHAC, V>;

impl<ECAH, EHAC, const V: u8> OneBot<ECAH, EHAC, V> {
    pub async fn start<E, A, R>(
        self: &Arc<Self>,
        ecah_config: ECAH::Config,
        ehac_config: EHAC::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        E: Static,
        A: Static,
        R: Static,
        ECAH: ECAHtrait<E, A, R, EHAC, V> + Static,
        EHAC: EHACtrait<E, A, R, ECAH, V> + Static,
    {
        let mut signal = self.signal.lock().await;
        if signal.is_none() {
            let (tx, _) = tokio::sync::broadcast::channel(1);
            *signal = Some(tx);
        } else {
            return Err(WalleError::AlreadyRunning);
        }
        let mut tasks = vec![];
        tasks.extend(self.ecah.ecah_start(self, ecah_config).await?.into_iter());
        tasks.extend(self.ehac.ehac_start(self, ehac_config).await?.into_iter());
        Ok(tasks)
    }
    pub fn handle_event<'a, E, A, R>(&'a self, event: E) -> impl Future<Output = ()> + 'a
    where
        EHAC: EHACtrait<E, A, R, ECAH, V> + Static,
    {
        self.ehac.handle_event(event, self)
    }
    pub fn handle_action<'a, E, A, R>(
        &'a self,
        action: A,
    ) -> impl Future<Output = WalleResult<R>> + 'a
    where
        R: Static,
        ECAH: ECAHtrait<E, A, R, EHAC, V> + Static,
    {
        self.ecah.handle_action(action, self)
    }
    pub async fn shutdown(&self) -> WalleResult<()> {
        let tx = self
            .signal
            .lock()
            .await
            .take()
            .ok_or(WalleError::NotRunning)?;
        tx.send(()).ok();
        Ok(())
    }
    pub async fn get_signal_rx(&self) -> WalleResult<tokio::sync::broadcast::Receiver<()>> {
        Ok(self
            .signal
            .lock()
            .await
            .as_ref()
            .ok_or(WalleError::NotRunning)?
            .subscribe())
    }
}
