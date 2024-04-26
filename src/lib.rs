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
pub use ah::ActionHandler;
mod eh;
use dashmap::DashMap;
pub use eh::EventHandler;
use prelude::{Bot, Echo, Selft};
use std::{collections::HashSet, sync::atomic::Ordering};
use tokio::{sync::mpsc, task::JoinHandle};
use tracing::info;

#[cfg(any(feature = "impl-obc", feature = "app-obc"))]
pub mod obc;
#[cfg(test)]
mod test;

use structs::Version;

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
    // Version
    pub version: Version,
}

use std::sync::{atomic::AtomicUsize, Arc, Mutex as StdMutex};
use tokio::sync::Mutex;

pub use crate::error::{WalleError, WalleResult};

impl<AH, EH> OneBot<AH, EH> {
    pub fn new(action_handler: AH, event_handler: EH, version: Version) -> Self {
        Self {
            action_handler,
            event_handler,
            signal: StdMutex::new(None),
            ah_tasks: Mutex::default(),
            eh_tasks: Mutex::default(),
            version,
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
    pub fn get_status<E, A, R>(&self) -> structs::Status
    where
        AH: ActionHandler<E, A, R> + Send + Sync,
    {
        structs::Status {
            good: self.signal.lock().unwrap().is_some(),
            bots: if let Some(map) = self.action_handler.get_bot_map() {
                map.bots
                    .iter()
                    .map(|item| Bot {
                        selft: item.key().clone(),
                        online: !item.value().1.is_empty(),
                    })
                    .collect()
            } else {
                vec![]
            },
        }
    }
    pub fn contains_bot<E, A, R>(&self, selft: &Selft) -> bool
    where
        AH: ActionHandler<E, A, R> + Send + Sync,
    {
        self.action_handler
            .get_bot_map()
            .and_then(|map| map.bots.get(selft))
            .map_or(false, |v| !v.value().1.is_empty())
    }
}

#[derive(Debug)]
pub struct BotMap<A> {
    /// 登记获取连接序列号
    pub(crate) conn_seq: AtomicUsize,
    /// 根据 bot 的 self 获取其 impl 字段和所有的 action_tx
    ///
    /// value: (implt, action_tx)
    pub(crate) bots: DashMap<Selft, (String, Vec<mpsc::UnboundedSender<Echo<A>>>)>,
    /// 根据连接序列号获取其 action_tx 和 所有 bot self
    ///
    /// value: (action_tx, selfts)
    pub(crate) conns: DashMap<usize, (mpsc::UnboundedSender<Echo<A>>, HashSet<Selft>)>,
}

impl<A> Default for BotMap<A> {
    fn default() -> Self {
        Self {
            conn_seq: AtomicUsize::default(),
            bots: DashMap::default(),
            conns: DashMap::default(),
        }
    }
}

impl<A> BotMap<A> {
    /// 登记一个新链接，返回新链接的 conn_seq，并返回一个接收 Echo<A> 的 Receiver
    fn new_connect(&self) -> (usize, mpsc::UnboundedReceiver<Echo<A>>) {
        let seq = self.conn_seq.fetch_add(1, Ordering::Relaxed);
        let (tx, rx) = mpsc::unbounded_channel();
        self.conns.insert(seq, (tx, HashSet::default()));
        (seq, rx)
    }
    /// 根据 conn_seq 关闭一个链接，并移除所有相关的 bot 的 action_tx
    fn connect_closs(&self, tx_seq: &usize) {
        if let Some(selfts) = self.conns.remove(tx_seq) {
            for selft in selfts.1 .1 {
                let mut bot = self.bots.get_mut(&selft).unwrap();
                bot.value_mut()
                    .1
                    .retain(|htx| !htx.same_channel(&selfts.1 .0));
                if bot.value().1.is_empty() {
                    drop(bot);
                    self.bots.remove(&selft);
                }
            }
        }
    }
    /// 更新一个连接的 bot 列表
    fn connect_update(&self, tx_seq: &usize, bots: Vec<Bot>, implt: &str) {
        let mut get = self.conns.get_mut(tx_seq).unwrap();
        let tx = get.0.clone();
        let selfts = &mut get.1;
        for bot in bots {
            match (bot.online, selfts.contains(&bot.selft)) {
                (true, false) => {
                    selfts.insert(bot.selft.clone());
                    self.bots
                        .entry(bot.selft.clone())
                        .or_insert((implt.to_string(), vec![]))
                        .1
                        .push(tx.clone());
                    info!(
                        target: WALLE_CORE,
                        "New Bot connected: {}-{}", bot.selft.platform, bot.selft.user_id
                    );
                }
                (false, true) => {
                    selfts.remove(&bot.selft);
                    if let Some(mut bots) = self.bots.get_mut(&bot.selft) {
                        bots.value_mut().1.retain(|htx| !htx.same_channel(&tx));
                        if bots.1.is_empty() {
                            drop(bots);
                            self.bots.remove(&bot.selft);
                        }
                    }
                    info!(
                        target: WALLE_CORE,
                        "Bot disconnected: {}-{}", bot.selft.platform, bot.selft.user_id
                    );
                }
                _ => {}
            }
        }
    }
    /// 获取一个 bot 的 action_tx
    fn get_bot_tx(&self, bot: &Selft) -> Option<Vec<mpsc::UnboundedSender<Echo<A>>>> {
        self.bots.get(bot).as_deref().cloned().map(|v| v.1)
    }
}

#[test]
fn test_bot_map() {
    let map = BotMap::<crate::action::Action>::default();
    let (seq, _) = map.new_connect();
    assert_eq!(seq, 0);
    let (seq, _) = map.new_connect();
    assert_eq!(seq, 1);
    assert_eq!(
        map.conns
            .iter()
            .map(|i| i.key().clone())
            .collect::<HashSet<_>>(),
        HashSet::from([1, 0])
    );
    let self0 = Selft {
        platform: "".to_owned(),
        user_id: "0".to_owned(),
    };
    let self1 = Selft {
        platform: "".to_owned(),
        user_id: "1".to_owned(),
    };
    map.connect_update(
        &0,
        vec![Bot {
            selft: self0.clone(),
            online: true,
        }],
        "",
    );
    assert_eq!(map.bots.get(&self0).unwrap().1.len(), 1);
    assert!(map.conns.get(&0).unwrap().value().1.len() == 1);
    assert!(map.bots.get(&self1).is_none());
    assert!(map.get_bot_tx(&self0).is_some());
    assert!(map.get_bot_tx(&self1).is_none());
}
