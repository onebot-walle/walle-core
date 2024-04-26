use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;

use super::OBC;
use crate::util::{EchoInner, EchoS, GetSelf, ProtocolItem};
use crate::{ActionHandler, EventHandler, OneBot};
use crate::{WalleError, WalleResult};

use dashmap::DashMap;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tracing::warn;

#[cfg(feature = "http")]
mod app_http;
#[cfg(feature = "websocket")]
mod app_ws;

pub(crate) type EchoMap<R> = Arc<DashMap<EchoS, oneshot::Sender<R>>>;

/// OneBotConnect 应用端实现
///
/// AppOBC impl ActionHandler 接收 Action 并外发处理
///
/// Event 泛型要求实现 Clone + SelfId trait
/// Action 泛型要求实现 SelfId + ActionType trait
pub struct AppOBC<A, R> {
    pub(crate) _block_meta_event: AtomicBool, //todo
    pub(crate) echos: EchoMap<R>,             // echo channel sender 暂存 Map
    pub(crate) seq: AtomicU64,                // 用于生成 echo
    pub(crate) _bots: Arc<crate::BotMap<A>>,  // Bot action channel map
}

impl<A, R> AppOBC<A, R> {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn block_meta_event(&self, b: bool) {
        self._block_meta_event.swap(b, Ordering::Relaxed);
    }
}

impl<A, R> Default for AppOBC<A, R> {
    fn default() -> Self {
        Self {
            _block_meta_event: AtomicBool::new(true),
            echos: Arc::new(DashMap::new()),
            seq: AtomicU64::default(),
            _bots: Arc::new(Default::default()),
        }
    }
}

impl<A, R> AppOBC<A, R> {
    pub(crate) fn next_seg(&self) -> EchoS {
        EchoS(Some(EchoInner::S(
            self.seq
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                .to_string(),
        )))
    }
}

impl<E, A, R> ActionHandler<E, A, R> for AppOBC<A, R>
where
    E: ProtocolItem + Clone + GetSelf,
    A: ProtocolItem + GetSelf,
    R: ProtocolItem,
{
    type Config = crate::config::AppConfig;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: crate::config::AppConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        let mut tasks = vec![];
        #[cfg(feature = "websocket")]
        {
            self.wsr(ob, config.websocket_rev, &mut tasks).await?;
            self.ws(ob, config.websocket, &mut tasks).await?;
        }
        #[cfg(feature = "http")]
        {
            self.webhook(ob, config.http_webhook, &mut tasks).await?;
            self.http(ob, config.http, &mut tasks).await?;
        }
        Ok(tasks)
    }
    async fn call<AH, EH>(&self, action: A, ob: &Arc<OneBot<AH, EH>>) -> WalleResult<R>
    where
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        match ob
            .action_handler
            .get_bot_map()
            .ok_or_else(|| WalleError::BotNotExist)?
            .get_bot_tx(&action.get_self())
        {
            Some(action_txs) => {
                let (tx, rx) = oneshot::channel();
                let seq = self.next_seg();
                self.echos.insert(seq.clone(), tx);
                action_txs
                    .first()
                    .unwrap() //todo
                    .send(seq.pack(action))
                    .map_err(|e| {
                        warn!(target: super::OBC, "send action error: {}", e);
                        WalleError::Other(e.to_string())
                    })?;
                match tokio::time::timeout(std::time::Duration::from_secs(10), rx).await {
                    Ok(Ok(res)) => Ok(res),
                    Ok(Err(e)) => {
                        warn!(target: super::OBC, "resp recv error: {:?}", e);
                        Err(WalleError::Other(e.to_string()))
                    }
                    Err(_) => {
                        warn!(target: super::OBC, "resp timeout");
                        Err(WalleError::Other("resp timeout".to_string()))
                    }
                }
            }
            None => {
                warn!(target: super::OBC, "bot not found");
                Err(WalleError::BotNotExist)
            }
        }
    }
    fn get_bot_map(&self) -> Option<&crate::BotMap<A>> {
        Some(&self._bots)
    }
    async fn before_call_event<AH, EH>(
        &self,
        event: E,
        _ob: &Arc<OneBot<AH, EH>>,
    ) -> WalleResult<E> {
        if self._block_meta_event.load(Ordering::Relaxed) {
            use core::any::Any;
            let event: Box<dyn Any> = Box::new(event.clone());
            if let Ok(ty) = event.downcast::<crate::event::Event>().map(|e| e.ty) {
                if &ty == "meta" {
                    return Err(WalleError::Other("blocked".to_string()));
                }
            }
        }
        Ok(event)
    }
}
