use std::{future::Future, pin::Pin, sync::Arc};

use crate::{
    action::Action,
    event::Event,
    prelude::{Selft, Status},
    resp::{resp_error, Resp, RespError},
    ActionHandler, EventHandler, GetSelfs, GetStatus, GetVersion, OneBot, WalleResult,
};

use self::db_trait::SqlDB;
mod db_trait;

pub struct DataBase<DB, H> {
    database: DB,
    handler: H,
}

use walle_macro::_TryFromAction as TryFromAction;

enum ActionOrResp {
    Action(Action),
    Resp(Resp),
}

#[derive(Debug, Clone, PartialEq, Eq, TryFromAction)]
pub enum DataBaseAction {
    GetEvent {
        id: String,
    },
    GetEvents {
        user_id: Option<String>,
        group_id: Option<String>,
        channel_id: Option<String>,
        guild_id: Option<String>,
        limit: u32,
        offset: u32,
    },
}

impl<DB: SqlDB, H: ActionHandler + Send + Sync> DataBase<DB, H> {
    pub async fn new(handler: H) -> Self {
        Self {
            database: DB::new().await,
            handler,
        }
    }
    async fn handle_action(&self, action: Action) -> ActionOrResp {
        match action.action.as_str() {
            "get_event" | "get_events" => {
                let action = match DataBaseAction::try_from(action) {
                    Ok(a) => a,
                    Err(e) => {
                        return ActionOrResp::Resp(resp_error::bad_param(e.to_string()).into())
                    }
                };
                ActionOrResp::Resp(match self._handle_action(action).await {
                    Ok(resp) => resp,
                    Err(e) => e.into(),
                })
            }
            _ => ActionOrResp::Action(action),
        }
    }
    async fn _handle_action(&self, _action: DataBaseAction) -> Result<Resp, RespError> {
        todo!()
    }
}

impl<DB: SqlDB, H: GetVersion> GetVersion for DataBase<DB, H> {
    fn get_version(&self) -> crate::prelude::Version {
        self.handler.get_version()
    }
}

impl<DB: SqlDB, H: GetStatus + Sync> GetStatus for DataBase<DB, H> {
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

impl<DB: SqlDB, H: GetSelfs> GetSelfs for DataBase<DB, H> {
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

impl<DB: SqlDB + Send + Sync, H: ActionHandler + Send + Sync> ActionHandler for DataBase<DB, H> {
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
            match self.handle_action(action).await {
                ActionOrResp::Action(a) => self.handler.call(a, ob).await,
                ActionOrResp::Resp(r) => Ok(r),
            }
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
            self.database.insert_event(&event).await;
            Ok(event)
        })
    }
}
