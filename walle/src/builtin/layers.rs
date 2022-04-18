use crate::{Handler, Session};
use std::future::Future;
use std::pin::Pin;
use walle_core::app::StandardArcBot;
use walle_core::{EventContent, MessageContent, MessageEvent, StandardEvent};

pub struct UserIdCheckedHanler<I> {
    pub user_id: String,
    pub inner: I,
}

impl<I> Handler<MessageContent> for UserIdCheckedHanler<I>
where
    I: Handler<MessageContent>,
{
    fn _match(&self, bot: &StandardArcBot, event: &MessageEvent) -> bool {
        if event.user_id() == self.user_id {
            return self.inner._match(bot, event);
        }
        false
    }

    fn handle<'a, 't>(
        &'a self,
        session: Session<MessageContent>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.inner.handle(session)
    }
}

impl<I> Handler<EventContent> for UserIdCheckedHanler<I>
where
    I: Handler<EventContent>,
{
    fn _match(&self, bot: &StandardArcBot, event: &StandardEvent) -> bool {
        if let EventContent::Message(ref c) = event.content {
            if c.user_id == self.user_id {
                return self.inner._match(bot, event);
            }
        }
        false
    }

    fn handle<'a, 't>(
        &'a self,
        session: Session<EventContent>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.inner.handle(session)
    }
}

pub fn user_id_layer<S, I>(user_id: S, inner: I) -> UserIdCheckedHanler<I>
where
    S: ToString,
{
    UserIdCheckedHanler {
        user_id: user_id.to_string(),
        inner,
    }
}

pub struct GroupIdCheckedHandler<I> {
    pub group_id: String,
    pub inner: I,
}

impl<I> Handler<MessageContent> for GroupIdCheckedHandler<I>
where
    I: Handler<MessageContent>,
{
    fn _match(&self, bot: &StandardArcBot, event: &MessageEvent) -> bool {
        if event.group_id() == Some(&self.group_id) {
            return self.inner._match(bot, event);
        }
        false
    }

    fn handle<'a, 't>(
        &'a self,
        session: Session<MessageContent>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.inner.handle(session)
    }
}

impl<I> Handler<EventContent> for GroupIdCheckedHandler<I>
where
    I: Handler<EventContent>,
{
    fn _match(&self, bot: &StandardArcBot, event: &StandardEvent) -> bool {
        if let EventContent::Message(ref c) = event.content {
            if c.ty.group_id() == Some(&self.group_id) {
                return self.inner._match(bot, event);
            }
        }
        false
    }

    fn handle<'a, 't>(
        &'a self,
        session: Session<EventContent>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.inner.handle(session)
    }
}

pub fn group_id_layer<S, I>(group_id: S, inner: I) -> GroupIdCheckedHandler<I>
where
    S: ToString,
{
    GroupIdCheckedHandler {
        group_id: group_id.to_string(),
        inner,
    }
}
