use crate::{box_rule, BoxedRule, Rule, Session};
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

pub fn user_id_layer<S, I>(user_id: S, inner: I) -> BoxedRule<UserIdChecker, I>
where
    S: ToString,
{
    box_rule(
        UserIdChecker {
            user_id: user_id.to_string(),
        },
        inner,
    )
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

pub fn group_id_layer<S, I>(group_id: S, inner: I) -> BoxedRule<GroupIdChecker, I>
where
    S: ToString,
{
    box_rule(
        GroupIdChecker {
            group_id: group_id.to_string(),
        },
        inner,
    )
}
