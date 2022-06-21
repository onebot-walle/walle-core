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
pub trait ActionHandler<E, A, R, OB> {
    type Config;
    async fn ah_start(
        &self,
        ob: &Arc<OB>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>;
    async fn handle_action(&self, action: A, ob: &OB) -> WalleResult<R>;
}

#[async_trait]
pub trait EventHandler<E, A, R, OB> {
    type Config;
    async fn eh_start(
        &self,
        ob: &Arc<OB>,
        config: Self::Config,
    ) -> WalleResult<Vec<JoinHandle<()>>>;
    async fn handle_event(&self, event: E, ob: &OB);
}

pub type ImplOneBot<E, AH, const V: u8> = OneBot<AH, obc::ImplOBC<E>, V>;
pub type AppOneBot<A, R, EH, const V: u8> = OneBot<obc::AppOBC<A, R>, EH, V>;

impl<E, A, R, AH, EH, const V: u8> EventHandler<E, A, R, Self> for OneBot<AH, EH, V>
where
    EH: EventHandler<E, A, R, Self> + Static,
{
    type Config = EH::Config;
    fn eh_start<'life0, 'life1, 'async_trait>(
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
        Self: 'async_trait,
    {
        self.event_handler.eh_start(ob, config)
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
        self.event_handler.handle_event(event, ob)
    }
}

impl<E, A, R, AH, EH, const V: u8> ActionHandler<E, A, R, Self> for OneBot<AH, EH, V>
where
    AH: ActionHandler<E, A, R, Self> + Static,
{
    type Config = AH::Config;
    fn ah_start<'life0, 'life1, 'async_trait>(
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
        Self: 'async_trait,
    {
        self.action_handler.ah_start(ob, config)
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
        self.action_handler.handle_action(action, ob)
    }
}

pub trait OneBotExt {
    fn get_signal_rx(&self) -> WalleResult<broadcast::Receiver<()>>;
    fn get_onebot_version(&self) -> u8;
}

impl<AH, EH, const V: u8> OneBotExt for OneBot<AH, EH, V> {
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
}

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
        AH: ActionHandler<E, A, R, Self> + Static,
        EH: EventHandler<E, A, R, Self> + Static,
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
                    .ah_start(self, ah_config)
                    .await?
                    .into_iter(),
            );
            tasks.extend(
                self.event_handler
                    .eh_start(self, eh_config)
                    .await?
                    .into_iter(),
            );
        } else {
            tasks.extend(
                self.event_handler
                    .eh_start(self, eh_config)
                    .await?
                    .into_iter(),
            );
            tasks.extend(
                self.action_handler
                    .ah_start(self, ah_config)
                    .await?
                    .into_iter(),
            );
        }
        Ok(tasks)
    }
    pub fn started(&self) -> bool {
        self.signal.lock().unwrap().is_some()
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
