use crate::{Handler, Session};
use std::future::Future;
use std::pin::Pin;

pub trait PreHandler<C> {
    fn pre_handle(&self, session: &mut Session<C>);
    fn layer<H>(self, handler: H) -> LayeredPreHandler<Self, H>
    where
        Self: Sized,
        H: Handler<C>,
    {
        LayeredPreHandler { pre: self, handler }
    }
}

pub struct LayeredPreHandler<P, H> {
    pub pre: P,
    pub handler: H,
}

impl<P, H, C> Handler<C> for LayeredPreHandler<P, H>
where
    P: PreHandler<C> + Sync,
    H: Handler<C> + Sync,
    C: 'static,
{
    fn _pre_handle(&self, session: &mut Session<C>) {
        self.pre.pre_handle(session);
    }
    fn handle<'a, 't>(
        &'a self,
        session: Session<C>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.handler.handle(session)
    }
}

pub struct PreHandleFn<I>(I);

impl<I, C> PreHandler<C> for PreHandleFn<I>
where
    I: Fn(&mut Session<C>) + Sync,
{
    fn pre_handle(&self, session: &mut Session<C>) {
        self.0(session);
    }
}

pub fn pre_handle_fn<I, C>(pre: I) -> PreHandleFn<I>
where
    I: Fn(&mut Session<C>) + Sync,
{
    PreHandleFn(pre)
}
