use std::sync::Arc;

use super::OBC;
use crate::event::Event;
use crate::util::ProtocolItem;
use crate::WalleResult;
use crate::{ActionHandler, EventHandler, OneBot};
use async_trait::async_trait;
use tokio::sync::broadcast;
use tokio::task::JoinHandle;

#[cfg(feature = "http")]
mod impl_http;
#[cfg(feature = "websocket")]
mod impl_ws;

/// OneBotConnect 实现端实现
///
/// ImplOBC impl EventHandler 接收 Event 并外发处理
///
/// ImplOBC 仅对 Event 泛型要求 Clone trait
pub struct ImplOBC<E> {
    pub self_id: std::sync::RwLock<String>,
    pub platform: String,
    pub implt: String,
    pub(crate) event_tx: tokio::sync::broadcast::Sender<E>,
    pub(crate) hb_tx: tokio::sync::broadcast::Sender<crate::event::Event>,
}

#[async_trait]
impl<E, A, R> EventHandler<E, A, R, 12> for ImplOBC<E>
where
    E: ProtocolItem + Clone,
    A: ProtocolItem,
    R: ProtocolItem,
{
    type Config = crate::config::ImplConfig;
    async fn start<AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH, 12>>,
        config: crate::config::ImplConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        AH: ActionHandler<E, A, R, 12> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, 12> + Send + Sync + 'static,
    {
        let mut tasks = vec![];
        #[cfg(feature = "websocket")]
        {
            self.ws(ob, config.websocket, &mut tasks).await?;
            self.wsr(ob, config.websocket_rev, &mut tasks).await?;
        }
        #[cfg(feature = "http")]
        {
            self.http(ob, config.http, &mut tasks).await?;
            self.webhook(ob, config.http_webhook, &mut tasks).await?;
        }
        if config.heartbeat.enabled {
            tasks.push(start_hb(
                &ob,
                self.self_id.read().unwrap().clone(),
                self.implt.clone(),
                self.platform.clone(),
                config.heartbeat.interval,
                self.hb_tx.clone(),
            ))
        }
        Ok(tasks)
    }
    async fn call<AH, EH>(&self, event: E, _ob: &Arc<OneBot<AH, EH, 12>>)
    where
        AH: ActionHandler<E, A, R, 12> + Send + Sync + 'static,
        EH: EventHandler<E, A, R, 12> + Send + Sync + 'static,
    {
        self.event_tx.send(event).ok();
    }
}

impl<E> ImplOBC<E> {
    pub fn new(self_id: String, r#impl: String, platform: String) -> Self
    where
        E: Clone,
    {
        let (event_tx, _) = tokio::sync::broadcast::channel(1024); //todo
        let (hb_tx, _) = tokio::sync::broadcast::channel(1024);
        Self {
            self_id: std::sync::RwLock::new(self_id),
            platform,
            implt: r#impl,
            event_tx,
            hb_tx,
        }
    }
    pub fn get_self_id(&self) -> String {
        self.self_id.read().unwrap().clone()
    }
    pub fn set_self_id(&self, self_id: &str) {
        *self.self_id.write().unwrap() = self_id.to_string();
    }
}

fn build_hb(self_id: &str, implt: &str, platform: &str, interval: u32) -> crate::event::Event {
    crate::event::Event {
        id: crate::util::new_uuid(),
        implt: implt.to_string(),
        platform: platform.to_string(),
        self_id: self_id.to_string(),
        time: crate::util::timestamp_nano_f64(),
        ty: "meta".to_string(),
        detail_type: "heartbeat".to_string(),
        sub_type: "".to_string(),
        extra: crate::extended_map! {
            "interval": interval,
            "status": { //todo
                "good": true,
                "online": true
            }
        },
    }
}

fn start_hb<AH, EH, const V: u8>(
    ob: &OneBot<AH, EH, V>,
    self_id: String,
    implt: String,
    platform: String,
    interval: u32,
    hb_tx: broadcast::Sender<Event>,
) -> JoinHandle<()> {
    let hb = build_hb(&self_id, &implt, &platform, interval);
    let hb_tx = Arc::new(hb_tx);
    let mut signal = ob.get_signal_rx().unwrap();
    tokio::spawn(async move {
        loop {
            if let Ok(_) = signal.try_recv() {
                break;
            }
            hb_tx.send(hb.clone()).ok();
            tokio::time::sleep(std::time::Duration::from_secs(interval as u64)).await;
        }
    })
}
