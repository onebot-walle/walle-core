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
pub struct OneBot<E, A, R, ECAH, EHAC, const V: u8> {
    pub ecah: ECAH,
    pub ehac: EHAC,

    phantom: std::marker::PhantomData<(E, A, R)>,
    // Some for running, None for stopped
    signal: Mutex<Option<broadcast::Sender<()>>>,
}

#[async_trait]
pub trait ECAHtrait<E, A, R, OB, C> {
    async fn ecah_start<C0>(&self, ob: &Arc<OB>, config: C) -> WalleResult<Vec<JoinHandle<()>>>
    where
        OB: EHACtrait<E, A, R, OB, C0> + OneBotExt;
    async fn handle_action(&self, id: &str, action: A, ob: &OB) -> WalleResult<R>;
}

#[async_trait]
pub trait EHACtrait<E, A, R, OB, C> {
    async fn ehac_start<C0>(&self, ob: &Arc<OB>, config: C) -> WalleResult<Vec<JoinHandle<()>>>
    where
        OB: ECAHtrait<E, A, R, OB, C0> + OneBotExt;
    async fn handle_event(&self, event: E, ob: &OB);
}

pub type ImplOneBot<E, A, R, ECAH, const V: u8> = OneBot<E, A, R, ECAH, obc::ImplOBC<E>, V>;
pub type AppOneBot<E, A, R, EHAC, const V: u8> = OneBot<E, A, R, obc::AppOBC<A, R>, EHAC, V>;

impl<E, A, R, C, ECAH, EHAC, const V: u8> ECAHtrait<E, A, R, Self, C>
    for OneBot<E, A, R, ECAH, EHAC, V>
where
    ECAH: ECAHtrait<E, A, R, Self, C>,
{
    fn ecah_start<'life0, 'life1, 'async_trait, C0>(
        &'life0 self,
        ob: &'life1 Arc<Self>,
        config: C,
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
        Self: EHACtrait<E, A, R, Self, C0> + OneBotExt + 'async_trait,
        C0: 'async_trait,
    {
        self.ecah.ecah_start(ob, config)
    }
    fn handle_action<'life0, 'life1, 'life2, 'async_trait>(
        &'life0 self,
        id: &'life1 str,
        action: A,
        ob: &'life2 Self,
    ) -> core::pin::Pin<
        Box<dyn core::future::Future<Output = WalleResult<R>> + core::marker::Send + 'async_trait>,
    >
    where
        'life0: 'async_trait,
        'life1: 'async_trait,
        'life2: 'async_trait,
        Self: 'async_trait,
    {
        self.ecah.handle_action(id, action, ob)
    }
}

impl<E, A, R, C, ECAH, EHAC, const V: u8> EHACtrait<E, A, R, Self, C>
    for OneBot<E, A, R, ECAH, EHAC, V>
where
    EHAC: EHACtrait<E, A, R, Self, C>,
{
    fn ehac_start<'life0, 'life1, 'async_trait, C0>(
        &'life0 self,
        ob: &'life1 Arc<Self>,
        config: C,
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
        Self: ECAHtrait<E, A, R, Self, C0> + OneBotExt + 'async_trait,
        C0: 'async_trait,
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

impl<E, A, R, ECAH, EHAC, const V: u8> OneBot<E, A, R, ECAH, EHAC, V>
where
    E: Static + Clone,
    A: Static,
    R: Static,
{
    pub async fn start<C0, C1>(
        self: &Arc<Self>,
        ecah_config: C0,
        ehac_config: C1,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        ECAH: ECAHtrait<E, A, R, Self, C0> + Static,
        EHAC: EHACtrait<E, A, R, Self, C1> + Static,
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
}

impl<E, A, R, ECAH, EHAC, const V: u8> OneBot<E, A, R, ECAH, EHAC, V> {
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
}

#[async_trait]
pub trait OneBotExt {
    fn get_onebot_version(&self) -> u8;
    async fn get_signal_rx(&self) -> WalleResult<tokio::sync::broadcast::Receiver<()>>;
}

#[async_trait]
impl<E, A, R, ECAH, EHAC, const V: u8> OneBotExt for OneBot<E, A, R, ECAH, EHAC, V>
where
    E: Static,
    A: Static,
    R: Static,
    ECAH: Static,
    EHAC: Static,
{
    fn get_onebot_version(&self) -> u8 {
        V
    }
    async fn get_signal_rx(&self) -> WalleResult<tokio::sync::broadcast::Receiver<()>> {
        Ok(self
            .signal
            .lock()
            .await
            .as_ref()
            .ok_or(WalleError::NotRunning)?
            .subscribe())
    }
}
