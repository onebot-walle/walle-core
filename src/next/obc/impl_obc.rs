use std::sync::Arc;

use crate::next::{ActionHandler, EventHandler, OneBotExt, Static};
use crate::utils::ProtocolItem;
use crate::WalleResult;
use async_trait::async_trait;
use tokio::task::JoinHandle;

/// OneBotConnect 实现端实现
///
/// ImplOBC impl EventHandler 接收 Event 并外发处理
///
/// ImplOBC 仅对 Event 泛型要求 Clone trait
pub struct ImplOBC<E> {
    pub self_id: std::sync::RwLock<String>,
    pub platform: String,
    pub r#impl: String,
    pub(crate) event_tx: tokio::sync::broadcast::Sender<E>,
    pub(crate) hb_tx: tokio::sync::broadcast::Sender<crate::StandardEvent>,
}

#[async_trait]
impl<E, A, R, OB> EventHandler<E, A, R, OB> for ImplOBC<E>
where
    E: ProtocolItem + Clone,
    A: ProtocolItem,
    R: ProtocolItem,
    OB: ActionHandler<E, A, R, OB> + OneBotExt + Static,
{
    type Config = crate::config::ImplConfig;
    async fn ehac_start(
        &self,
        ob: &Arc<OB>,
        config: crate::config::ImplConfig,
    ) -> WalleResult<Vec<JoinHandle<()>>> {
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
        Ok(tasks)
    }
    async fn handle_event(&self, event: E, _ob: &OB) {
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
            r#impl,
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

impl<C> ImplOBC<crate::BaseEvent<C>> {
    pub fn new_event_with_time(
        &self,
        time: f64,
        content: C,
        self_id: String,
    ) -> crate::BaseEvent<C> {
        crate::BaseEvent {
            id: crate::utils::new_uuid(),
            r#impl: self.r#impl.clone(),
            platform: self.platform.clone(),
            self_id,
            time,
            content,
        }
    }

    pub fn new_event(&self, content: C, self_id: String) -> crate::BaseEvent<C> {
        self.new_event_with_time(crate::utils::timestamp_nano_f64(), content, self_id)
    }
}
