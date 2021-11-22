use actix::{dev::MessageResponse, Actor, Context, Handler, Message};
use std::marker::PhantomData;
use walle_core::{action_resp::FromStandard, Action as ObAction, ActionResp as ObActionResp};

pub struct CustomOneBot {}

pub struct Action<A, R> {
    pub inner: A,
    _result: PhantomData<R>,
}

impl<A, R> Message for Action<A, R>
where
    R: std::fmt::Debug + 'static,
{
    type Result = ActionResp<R>;
}

#[derive(Debug)]
pub struct ActionResp<R: std::fmt::Debug> {
    pub inner: ObActionResp<R>,
}

impl Actor for CustomOneBot {
    type Context = Context<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {
        println!("Walle-actix-OneBot started");
    }
}

impl<A, R> MessageResponse<CustomOneBot, Action<A, R>> for ActionResp<R>
where
    R: std::fmt::Debug + 'static,
{
    fn handle(
        self,
        _: &mut Context<CustomOneBot>,
        tx: Option<actix::dev::OneshotSender<ActionResp<R>>>,
    ) {
        if let Some(tx) = tx {
            tx.send(self).unwrap();
        }
    }
}

impl<A, R> Handler<Action<A, R>> for CustomOneBot
where
    R: MessageResponse<CustomOneBot, Action<A, R>> + FromStandard + std::fmt::Debug + 'static,
{
    type Result = ActionResp<R>;
    fn handle(&mut self, _: Action<A, R>, _: &mut Self::Context) -> Self::Result {
        ActionResp {
            inner: ObActionResp::unsupported_action(),
        }
    }
}
