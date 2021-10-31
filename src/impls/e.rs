use super::CustomOneBot;
use crate::{utils::timestamp, Event, EventContent, RUNNING};
use std::sync::{atomic::Ordering, Arc};

impl<A, R> CustomOneBot<EventContent, A, R>
where
    A: serde::de::DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: serde::Serialize + std::fmt::Debug + Send + 'static,
{
    pub fn new_event(&self, id: String, content: EventContent) -> Event {
        crate::event::CustomEvent {
            id,
            r#impl: self.r#impl.clone(),
            platform: self.platform.clone(),
            self_id: self.self_id.clone(),
            time: crate::utils::timestamp(),
            content,
        }
    }

    pub fn start_heartbeat(&self, ob: Arc<Self>) {
        let mut interval = self.config.heartbeat.interval;
        if interval <= 0 {
            interval = 4;
        }
        tokio::spawn(async move {
            loop {
                if ob.status.load(Ordering::SeqCst) != RUNNING {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(interval as u64)).await;
                let t = timestamp();
                let hb = ob.new_event(
                    format!("{}", t),
                    EventContent::Meta(crate::event::Meta::Heartbeat {
                        interval,
                        status: ob.get_status(),
                        sub_type: "".to_owned(),
                    }),
                );
                match ob.send_event(hb) {
                    _ => {}
                };
            }
        });
    }
}
