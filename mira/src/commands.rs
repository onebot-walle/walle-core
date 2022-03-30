use tracing::{info, warn};
use walle_core::{Message, MessageBuild};

impl crate::Cache {
    pub(crate) async fn send_message(&self, msg: &str) {
        if !self.user_id.is_empty() {
            for (_, bot) in self.cli.get_bots().await.iter() {
                let resp = bot
                    .send_message(
                        "private".to_owned(),
                        None,
                        Some(self.user_id.clone()),
                        Message::new().text(msg.to_owned()),
                        [].into(),
                    )
                    .await
                    .unwrap();
                info!("{:?}", resp);
            }
        } else if !self.group_id.is_empty() {
            for (_, bot) in self.cli.get_bots().await.iter() {
                let resp = bot
                    .send_message(
                        "group".to_owned(),
                        Some(self.group_id.clone()),
                        None,
                        Message::new().text(msg.to_owned()),
                        [].into(),
                    )
                    .await
                    .unwrap();
                info!("{:?}", resp);
            }
        } else {
            warn!("no group or user id is setted");
        }
    }
}
