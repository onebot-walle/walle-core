use abras_onebot::tokio::task::JoinHandle;
use abras_onebot::{Event, EventContent, Message, OneBot, VersionContent};

use std::collections::HashMap;
use std::sync::Arc;

static NAME: &str = "Recruit";
static PLATFORM: &str = "shell";
static VERSION: &str = "0.1.0";
static ONEBOT_VERSION: &str = "12";

pub(crate) struct Bot {
    pub inner: Arc<OneBot>,
    event_count: u64,
    join_handle: Option<JoinHandle<()>>,
}

#[derive(Default)]
pub(crate) struct Bots {
    inner: HashMap<String, Bot>,
}

impl Bot {
    pub(crate) fn new(self_id: String, config: abras_onebot::Config) -> Self {
        Bot {
            inner: Arc::new(OneBot::new(
                NAME.to_owned(),
                PLATFORM.to_owned(),
                self_id,
                config,
                Arc::new(super::handle::Handler),
            )),
            event_count: 0,
            join_handle: None,
        }
    }

    fn build_private_event(&self, self_id: String, bot_id: String, message: Message) -> Event {
        self.inner.new_events(
            format!("{}", self.event_count),
            self_id,
            EventContent::new_message_content(
                "private".to_owned(),
                "".to_owned(),
                message,
                "".to_owned(),
                bot_id,
                None,
            ),
        )
    }

    fn run(&mut self) {
        if self.join_handle.is_none() {
            let bot = self.inner.clone();
            self.join_handle = Some(abras_onebot::tokio::spawn(async move {
                bot.run().await.unwrap();
            }));
        }
    }
}

impl Bots {
    pub(crate) async fn add_bot(&mut self, bot_id: String, mut bot: Bot) -> Option<Bot> {
        bot.run();
        self.inner.insert(bot_id, bot)
    }

    pub(crate) async fn remove_bot(&mut self, bot_id: &str) -> Option<Bot> {
        self.inner.remove(bot_id)
    }

    pub(crate) async fn send_private_message(
        &mut self,
        bot_id: String,
        self_id: &str,
        message: Message,
    ) -> Option<String> {
        if let Some(bot) = self.inner.get(&bot_id) {
            let e = bot.build_private_event(self_id.to_owned(), bot_id, message);
            let seq = e.id.clone();
            bot.inner.send_event(e);
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
