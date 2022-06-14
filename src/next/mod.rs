use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::task::JoinHandle;

use crate::{WalleError, WalleResult};

pub mod obc;

type ActionContext<A, R> = (String, A, mpsc::UnboundedSender<R>);
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
    async fn start(
        &self,
        ob: &Arc<OB>,
        config: C,
    ) -> WalleResult<(Vec<JoinHandle<()>>, broadcast::Receiver<E>)>;
    async fn handle(&self, action_context: ActionContext<A, R>, ob: &OB);
}

#[async_trait]
pub trait EHACtrait<E, A, R, OB, C> {
    async fn start(
        &self,
        ob: &Arc<OB>,
        config: C,
    ) -> WalleResult<(
        Vec<JoinHandle<()>>,
        mpsc::UnboundedReceiver<ActionContext<A, R>>,
    )>;
    async fn handle(&self, event: E, ob: &OB);
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
        let (joins, mut event_rx) = self.ecah.start(self, ecah_config).await?;
        tasks.extend(joins.into_iter());
        let (joins, mut action_rx) = self.ehac.start(self, ehac_config).await?;
        tasks.extend(joins.into_iter());
        let ob = self.clone();
        tasks.push(tokio::spawn(async move {
            while let Ok(event) = event_rx.recv().await {
                ob.ehac.handle(event, &ob).await;
            }
        }));
        let ob = self.clone();
        tasks.push(tokio::spawn(async move {
            while let Some(ac) = action_rx.recv().await {
                ob.ecah.handle(ac, &ob).await;
            }
        }));
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
