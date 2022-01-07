use crate::prelude::*;
use async_trait::async_trait;
use tokio::sync::mpsc::UnboundedSender;
use walle_core::{ActionHandler, EventHandler};

pub(crate) struct Collector {
    pub(crate) event_tx: UnboundedSender<v12Event>,
    pub(crate) action_tx: UnboundedSender<(v12Action, tokio::sync::oneshot::Sender<v12Resp>)>,
}

#[async_trait]
impl ActionHandler<v12Action, v12Resp> for Collector {
    async fn handle(&self, action: v12Action) -> v12Resp {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.action_tx.send((action, tx)).unwrap();
        rx.await.unwrap()
    }
}

#[async_trait]
impl EventHandler<v12Event, v12Action, v12Resp> for Collector {
    async fn handle(&self, _: ArcBot<v12Action, v12Resp>, event: v12Event) {
        self.event_tx.send(event).unwrap();
    }
}
