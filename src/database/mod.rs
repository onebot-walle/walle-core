use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    action::Action,
    event::Event,
    prelude::{Selft, Status},
    resp::Resp,
    ActionHandler, EventHandler, GetSelfs, GetStatus, GetVersion, OneBot, WalleResult,
};

pub struct DataBase<H> {
    _database: (),
    handler: H,
}

impl<H: ActionHandler + Send + Sync> DataBase<H> {
    pub fn new(handler: H) -> Self {
        Self {
            _database: (),
            handler,
        }
    }
}

impl<H: GetVersion> GetVersion for DataBase<H> {
    fn get_version(&self) -> crate::prelude::Version {
        self.handler.get_version()
    }
}

impl<H: GetStatus + Sync> GetStatus for DataBase<H> {
    fn is_good<'a, 't>(&'a self) -> Pin<Box<dyn Future<Output = bool> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.handler.is_good()
    }
    fn get_status<'a, 't>(&'a self) -> Pin<Box<dyn Future<Output = Status> + Send + 't>>
    where
        Self: Sized,
        'a: 't,
        Self: core::marker::Sync + 't,
    {
        self.handler.get_status()
    }
}

impl<H: GetSelfs> GetSelfs for DataBase<H> {
    fn get_impl<'a, 'b, 't>(
        &'a self,
        selft: &'b Selft,
    ) -> Pin<Box<dyn Future<Output = String> + Send + 't>>
    where
        'a: 't,
        'b: 't,
        Self: 't,
    {
        self.handler.get_impl(selft)
    }
    fn get_selfs<'a, 't>(&'a self) -> Pin<Box<dyn Future<Output = Vec<Selft>> + Send + 't>>
    where
        'a: 't,
        Self: 't,
    {
        self.handler.get_selfs()
    }
}

impl<H: ActionHandler + Send + Sync> ActionHandler for DataBase<H> {
    type Config = H::Config;
    fn start<'a, 'b, 't, AH, EH>(
        &'a self,
        ob: &'b Arc<OneBot<AH, EH>>,
        config: Self::Config,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Vec<tokio::task::JoinHandle<()>>>> + Send + 't>>
    where
        AH: ActionHandler + Send + Sync + 'static,
        EH: EventHandler + Send + Sync + 'static,
        AH: 't,
        EH: 't,
        'a: 't,
        'b: 't,
        Self: 't,
    {
        self.handler.start(ob, config)
    }
    fn call<'a, 'b, 't, AH, EH>(
        &'a self,
        action: Action,
        ob: &'b Arc<OneBot<AH, EH>>,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Resp>> + Send + 't>>
    where
        AH: ActionHandler + Send + Sync + 'static,
        EH: EventHandler + Send + Sync + 'static,
        AH: 't,
        EH: 't,
        'a: 't,
        'b: 't,
        Self: 't,
    {
        Box::pin(async move {
            //todo
            self.handler.call(action, ob).await
        })
    }
    fn before_call_event<'a, 'b, 't, AH, EH>(
        &'a self,
        event: Event,
        _ob: &'b Arc<OneBot<AH, EH>>,
    ) -> Pin<Box<dyn Future<Output = WalleResult<Event>> + Send + 't>>
    where
        Event: Send + 'static,
        AH: ActionHandler + Send + Sync + 'static,
        EH: EventHandler + Send + Sync + 'static,
        AH: 't,
        EH: 't,
        'a: 't,
        'b: 't,
        Self: 't,
    {
        Box::pin(async move {
            //todo
            Ok(event)
        })
    }
}
