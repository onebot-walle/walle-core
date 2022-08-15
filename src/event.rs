use crate::{
    prelude::WalleError,
    structs::{Selft, Status},
    util::{GetSelf, PushToValueMap, Value, ValueMap, ValueMapExt},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    #[serde(rename = "impl")]
    pub implt: String,
    pub time: f64,
    #[serde(rename = "type")]
    pub ty: String,
    pub detail_type: String,
    pub sub_type: String,
    #[serde(flatten)]
    pub extra: ValueMap,
}

impl Event {
    pub fn selft(&self) -> Option<Selft> {
        self.extra.get_downcast("self").ok()
    }
    pub fn self_id(&self) -> Option<String> {
        self.selft().map(|s| s.user_id)
    }
    pub fn platform(&self) -> Option<String> {
        self.selft().map(|s| s.platform)
    }
}

impl ValueMapExt for Event {
    fn get_downcast<T>(&self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.extra.get_downcast(key)
    }
    fn remove_downcast<T>(&mut self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.extra.remove_downcast(key)
    }
    fn try_get_downcast<T>(&self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.extra.try_get_downcast(key)
    }
    fn try_remove_downcast<T>(&mut self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.extra.try_remove_downcast(key)
    }
    fn push<T>(&mut self, value: T)
    where
        T: PushToValueMap,
    {
        value.push_to(&mut self.extra)
    }
}

impl From<Event> for Value {
    fn from(e: Event) -> Self {
        let mut map = e.extra;
        map.insert("id".to_string(), e.id.into());
        map.insert("impl".to_string(), e.implt.into());
        map.insert("time".to_string(), e.time.into());
        map.insert("type".to_string(), e.ty.into());
        map.insert("detail_type".to_string(), e.detail_type.into());
        map.insert("sub_type".to_string(), e.sub_type.into());
        Value::Map(map)
    }
}

impl TryFrom<Value> for Event {
    type Error = WalleError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Map(mut map) = value {
            Ok(Self {
                id: map.remove_downcast("id")?,
                implt: map.remove_downcast("impl")?,
                time: map.remove_downcast("time")?,
                ty: map.remove_downcast("type")?,
                detail_type: map.remove_downcast("detail_type")?,
                sub_type: map.remove_downcast("sub_type")?,
                extra: map,
            })
        } else {
            Err(WalleError::ValueTypeNotMatch(
                "map".to_string(),
                format!("{:?}", value),
            ))
        }
    }
}

impl GetSelf for Event {
    fn get_self(&self) -> Selft {
        self.selft().unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseEvent<T = (), D = (), S = (), P = (), I = ()> {
    pub id: String,
    pub time: f64,
    pub implt: I,
    pub platform: P,
    pub ty: T,
    pub detail_type: D,
    pub sub_type: S,
    pub extra: ValueMap,
}

impl<T, D, S, P, I> From<BaseEvent<T, D, S, P, I>> for Event
where
    T: TypeDeclare + PushToValueMap,
    D: DetailTypeDeclare + PushToValueMap,
    S: SubTypeDeclare + PushToValueMap,
    P: PlatformDeclare + PushToValueMap,
    I: ImplDeclare + PushToValueMap,
{
    fn from(mut event: BaseEvent<T, D, S, P, I>) -> Self {
        Self {
            id: event.id,
            implt: event.implt.implt().to_string(),
            time: event.time,
            ty: event.ty.ty().to_string(),
            detail_type: event.detail_type.detail_type().to_string(),
            sub_type: event.sub_type.sub_type().to_string(),
            extra: {
                let map = &mut event.extra;
                event.implt.push_to(map);
                event.platform.push_to(map);
                event.ty.push_to(map);
                event.detail_type.push_to(map);
                event.sub_type.push_to(map);
                event.extra
            },
        }
    }
}

pub fn new_event<T, D, S, P, I>(
    id: String,
    time: f64,
    ty: T,
    detail_type: D,
    sub_type: S,
    platform: P,
    implt: I,
    extra: ValueMap,
) -> BaseEvent<T, D, S, P, I> {
    BaseEvent::<T, D, S, P, I> {
        id,
        time,
        ty,
        detail_type,
        sub_type,
        platform,
        implt,
        extra,
    }
}

impl<T, D, S, P, I> TryFrom<Event> for BaseEvent<T, D, S, P, I>
where
    T: for<'a> TryFrom<&'a mut Event, Error = WalleError> + TypeDeclare,
    D: for<'a> TryFrom<&'a mut Event, Error = WalleError> + DetailTypeDeclare,
    S: for<'a> TryFrom<&'a mut Event, Error = WalleError> + SubTypeDeclare,
    I: for<'a> TryFrom<&'a mut Event, Error = WalleError> + ImplDeclare,
    P: for<'a> TryFrom<&'a mut Event, Error = WalleError> + PlatformDeclare,
{
    type Error = WalleError;
    fn try_from(mut event: Event) -> Result<Self, Self::Error> {
        if !T::check(&event) {
            return Err(WalleError::DeclareNotMatch("type", event.ty.clone()));
        } else if !D::check(&event) {
            return Err(WalleError::DeclareNotMatch(
                "detail_type",
                event.detail_type.clone(),
            ));
        } else if !S::check(&event) {
            return Err(WalleError::DeclareNotMatch(
                "sub_type",
                event.sub_type.clone(),
            ));
        } else if !P::check(&event) {
            return Err(WalleError::DeclareNotMatch(
                "platform",
                event.platform().unwrap_or_default(),
            ));
        } else if !I::check(&event) {
            return Err(WalleError::DeclareNotMatch("impl", event.implt.clone()));
        }
        Ok(Self {
            ty: T::try_from(&mut event)?,
            detail_type: D::try_from(&mut event)?,
            sub_type: S::try_from(&mut event)?,
            implt: I::try_from(&mut event)?,
            platform: P::try_from(&mut event)?,
            id: event.id,
            time: event.time,
            extra: event.extra,
        })
    }
}

pub trait ImplDeclare {
    fn implt(&self) -> &'static str {
        ""
    }
    fn check(_event: &Event) -> bool {
        true
    }
}

pub trait PlatformDeclare {
    fn platform(&self) -> &'static str {
        ""
    }
    fn check(_event: &Event) -> bool {
        true
    }
}

pub trait SubTypeDeclare {
    fn sub_type(&self) -> &'static str {
        ""
    }
    fn check(_event: &Event) -> bool {
        true
    }
}

pub trait DetailTypeDeclare {
    fn detail_type(&self) -> &'static str {
        ""
    }
    fn check(_event: &Event) -> bool {
        true
    }
}

pub trait TypeDeclare {
    fn ty(&self) -> &'static str {
        ""
    }
    fn check(_event: &Event) -> bool {
        true
    }
}

impl TypeDeclare for () {}
impl DetailTypeDeclare for () {}
impl SubTypeDeclare for () {}
impl PlatformDeclare for () {}
impl ImplDeclare for () {}
impl PushToValueMap for () {}
impl TryFrom<&mut Event> for () {
    type Error = WalleError;
    fn try_from(_: &mut Event) -> Result<Self, Self::Error> {
        Ok(())
    }
}

use walle_macro::{_OneBot as OneBot, _PushToValueMap as PushToValueMap};

#[derive(Debug, Clone, PartialEq, OneBot, PushToValueMap)]
#[event(type)]
pub struct Message {
    pub selft: Selft,
    pub message_id: String,
    pub message: crate::segment::Segments,
    pub alt_message: String,
    pub user_id: String,
}
pub type MessageEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Message, D, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(type)]
pub struct Notice {
    pub selft: Selft,
}
pub type NoticeEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Notice, D, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(type)]
pub struct Request {
    pub selft: Selft,
}
pub type RequestEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Request, D, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(type)]
pub struct Meta;
pub type MetaEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Meta, D, S, P, I>;

#[derive(Debug, Clone, OneBot, PushToValueMap)]
#[event(type)]
pub enum EventType {
    Meta,
    Message(Message),
    Request(Request),
    Notice(Notice),
}

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct Private;
pub type PrivateMessageEvent<S = (), P = (), I = ()> = BaseEvent<Message, Private, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct Group {
    pub group_id: String,
}
pub type GroupMessageEvent<S = (), P = (), I = ()> = BaseEvent<Message, Group, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct Heartbeat {
    pub interval: u32,
    pub status: crate::structs::Status,
}
pub type HeartbeatEvent<S = (), P = (), I = ()> = BaseEvent<Meta, Heartbeat, S, P, I>;

pub type StatusUpdateEvent<S = (), P = (), I = ()> = BaseEvent<Meta, Status, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct GroupMemberIncrease {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GroupMemberIncreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMemberIncrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct GroupMemberDecrease {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GroupMemberDecreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMemberDecrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct GroupMessageDelete {
    pub group_id: String,
    pub message_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GroupMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMessageDelete, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct FriendIncrease {
    pub user_id: String,
}
pub type FriendIncreaseEvent<S = (), P = (), I = ()> = BaseEvent<Notice, FriendIncrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct FriendDecrease {
    pub user_id: String,
}
pub type FriendDecreaseEvent<S = (), P = (), I = ()> = BaseEvent<Notice, FriendDecrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct PrivateMessageDelete {
    pub message_id: String,
    pub user_id: String,
}
pub type PrivateMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, PrivateMessageDelete, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct GuildMemberIncrease {
    pub guild_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GuildMemberIncreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GuildMemberIncrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct GuildMemberDecrease {
    pub guild_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GuildMemberDecreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GuildMemberDecrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct ChannelMessageDelete {
    pub guild_id: String,
    pub channel_id: String,
    pub user_id: String,
    pub operator_id: String,
    pub message_id: String,
}
pub type ChannelMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, ChannelMessageDelete, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct ChannelCreate {
    pub guild_id: String,
    pub channel_id: String,
    pub operator_id: String,
}
pub type ChannelCreateEvent<S = (), P = (), I = ()> = BaseEvent<Notice, ChannelCreate, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[event(detail_type)]
pub struct ChannelDelete {
    pub guild_id: String,
    pub channel_id: String,
    pub operator_id: String,
}
pub type ChannelDeleteEvent<S = (), P = (), I = ()> = BaseEvent<Notice, ChannelDelete, S, P, I>;

#[derive(Debug, Clone, OneBot)]
#[event(detail_type)]
pub enum MessageDeatilTypes {
    Group(Group),
    Private(Private),
}
