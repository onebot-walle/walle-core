use std::sync::Arc;

use async_trait::async_trait;
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
pub trait ECAHtrait<E, A, R, OB> {
    type Config;
    async fn ecah_start(
        &self,
        ob: &Arc<OB>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        OB: EHACtrait<E, A, R, OB>;
    async fn handle_action(&self, action: A, ob: &OB) -> WalleResult<R>;
}

#[async_trait]
pub trait EHACtrait<E, A, R, OB> {
    type Config;
    async fn ehac_start(
        &self,
        ob: &Arc<OB>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        OB: ECAHtrait<E, A, R, OB>;
    async fn handle_event(&self, event: E, ob: &OB);
}

pub type ImplOneBot<E, ECAH, const V: u8> = OneBot<ECAH, obc::ImplOBC<E>, V>;
pub type AppOneBot<A, R, EHAC, const V: u8> = OneBot<obc::AppOBC<A, R>, EHAC, V>;

impl<E, A, R, ECAH, EHAC, const V: u8> ECAHtrait<E, A, R, Self> for OneBot<ECAH, EHAC, V>
where
    ECAH: ECAHtrait<E, A, R, Self>,
{
    type Config = ECAH::Config;
    fn ecah_start<'life0, 'life1, 'async_trait>(
        &'life0 self,
        ob: &'life1 Arc<Self>,
        config: Self::Config,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = WalleResult<Vec<JoinHandle<()>>>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: EHACtrait<E, A, R, Self> + 'async_trait,
    {
        self.ecah.ecah_start(ob, config)
    }
    fn handle_action<'life0, 'life1, 'async_trait>(
        &'life0 self,
        action: A,
        ob: &'life1 Self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = WalleResult<R>> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.ecah.handle_action(action, ob)
    }
}

impl<E, A, R, ECAH, EHAC, const V: u8> EHACtrait<E, A, R, Self> for OneBot<ECAH, EHAC, V>
where
    EHAC: EHACtrait<E, A, R, Self>,
{
    type Config = EHAC::Config;
    fn ehac_start<'life0, 'life1, 'async_trait>(
        &'life0 self,
        ob: &'life1 Arc<Self>,
        config: Self::Config,
    ) -> core::pin::Pin<
        Box<
            dyn core::future::Future<Output = WalleResult<Vec<JoinHandle<()>>>>
                + core::marker::Send
                + 'async_trait,
        >,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: ECAHtrait<E, A, R, Self> + 'async_trait,
    {
        self.ehac.ehac_start(ob, config)
    }
    fn handle_event<'life0, 'life1, 'async_trait>(
        &'life0 self,
        event: E,
        ob: &'life1 Self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = ()> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        Self: 'async_trait,
    {
        self.ehac.handle_event(event, ob)
    }
}

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
        ECAH: ECAHtrait<E, A, R, Self> + Static,
        EHAC: EHACtrait<E, A, R, Self> + Static,
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
