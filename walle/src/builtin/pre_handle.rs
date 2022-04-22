use walle_core::MessageContent;

use crate::{PreHandler, Session};

pub struct StripPrefix {
    pub prefix: String,
}

impl PreHandler<MessageContent> for StripPrefix {
    fn pre_handle(&self, session: &mut Session<MessageContent>) {
        let _ = session.event.content.alt_message.strip_prefix(&self.prefix);
    }
}

pub fn strip_prefix<S>(prefix: S) -> StripPrefix
where
    S: ToString,
{
    StripPrefix {
        prefix: prefix.to_string(),
    }
}
