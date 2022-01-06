use std::collections::HashMap;
use std::sync::Arc;
use walle_core::{impls::OneBot, resp::VersionContent, Event, EventContent, ImplConfig, Message};

static NAME: &str = "Recruit";
static PLATFORM: &str = "shell";
static VERSION: &str = "0.1.0";
static ONEBOT_VERSION: &str = "12";

pub(crate) struct Bot {
    pub inner: Arc<OneBot>,
    // event_count: u64,
}

#[derive(Default)]
pub(crate) struct Bots {
    inner: HashMap<String, Bot>,
}

impl Bot {
    pub(crate) fn new(self_id: String, config: ImplConfig) -> Self {
        Bot {
            inner: OneBot::new(
                NAME.to_owned(),
                PLATFORM.to_owned(),
                self_id,
                config,
                Arc::new(super::handle::Handler),
            )
            .arc(),
            // event_count: 0,
        }
    }

    #[allow(unused)]
    fn build_private_event(&self, bot_id: String, message: Message, alt_message: String) -> Event {
        self.inner.new_event(EventContent::private(
            "".to_owned(),
            message,
            alt_message,
            bot_id,
        ))
    }
}

impl Bots {
    pub(crate) async fn add_bot(&mut self, bot_id: String, bot: Bot) -> Option<Bot> {
        bot.inner.run().await.unwrap();
        self.inner.insert(bot_id, bot)
    }

    #[allow(unused)]
    pub(crate) async fn send_private_message(
        &mut self,
        bot_id: String,
        message: Message,
        alt_message: String,
    ) -> Option<String> {
        if let Some(bot) = self.inner.get(&bot_id) {
            let e = bot.build_private_event(bot_id, message, alt_message);
            let seq = e.id.clone();
            match bot.inner.send_event(e) {
                _ => {}
            }
            Some(seq)
        } else {
            None
        }
    }
}

pub fn version() -> VersionContent {
    VersionContent {
        r#impl: NAME.to_owned(),
        platform: PLATFORM.to_owned(),
        version: VERSION.to_owned(),
        onebot_version: ONEBOT_VERSION.to_owned(),
    }
}
