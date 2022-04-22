use crate::{Rule, Session};
use walle_core::{EventContent, MessageContent};

pub struct UserIdChecker {
    pub user_id: String,
}

impl Rule<MessageContent> for UserIdChecker {
    fn rule(&self, session: &Session<MessageContent>) -> bool {
        session.event.user_id() == self.user_id
    }
}

impl Rule<EventContent> for UserIdChecker {
    fn rule(&self, session: &Session<EventContent>) -> bool {
        if let EventContent::Message(ref c) = session.event.content {
            c.user_id == self.user_id
        } else {
            false
        }
    }
}

pub fn user_id_check<S>(user_id: S) -> UserIdChecker
where
    S: ToString,
{
    UserIdChecker {
        user_id: user_id.to_string(),
    }
}

pub struct GroupIdChecker {
    pub group_id: String,
}

impl Rule<MessageContent> for GroupIdChecker {
    fn rule(&self, session: &Session<MessageContent>) -> bool {
        session.event.group_id() == Some(&self.group_id)
    }
}

impl Rule<EventContent> for GroupIdChecker {
    fn rule(&self, session: &Session<EventContent>) -> bool {
        if let EventContent::Message(ref c) = session.event.content {
            if c.ty.group_id() == Some(&self.group_id) {
                return true;
            }
        }
        false
    }
}

pub fn group_id_check<S>(group_id: S) -> GroupIdChecker
where
    S: ToString,
{
    GroupIdChecker {
        group_id: group_id.to_string(),
    }
}

pub fn start_with(word: &str) -> impl Rule<MessageContent> {
    use crate::rule_fn;
    let word = word.to_string();
    rule_fn(move |session: &Session<MessageContent>| {
        session.event.content.alt_message.starts_with(&word)
    })
}
