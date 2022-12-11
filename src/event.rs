//! Event 相关模型定义

use crate::{
    prelude::{WalleError, WalleResult},
    structs::Selft,
    util::{GetSelf, PushToValueMap, Value, ValueMap, ValueMapExt},
};

use serde::{Deserialize, Serialize};

/// 标准 Event 模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    pub time: f64,
    #[serde(rename = "type")]
    pub ty: String,
    pub detail_type: String,
    pub sub_type: String,
    #[serde(flatten)]
    pub extra: ValueMap,
}

pub trait ToEvent<T>: PushToValueMap {
    fn ty(&self) -> &'static str;
}

pub trait TryFromEvent<T>: Sized {
    fn try_from_event_mut(event: &mut Event, implt: &str) -> WalleResult<Self>;
    fn try_from_event(mut event: Event, implt: &str) -> WalleResult<Self> {
        Self::try_from_event_mut(&mut event, implt)
    }
}

#[doc(hidden)]
pub struct TypeLevel;
#[doc(hidden)]
pub struct DetailTypeLevel;
#[doc(hidden)]
pub struct SubTypeLevel;
#[doc(hidden)]
pub struct PlatformLevel;
#[doc(hidden)]
pub struct ImplLevel;

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

impl From<Event> for Value {
    fn from(e: Event) -> Self {
        let mut map = e.extra;
        map.insert("id".to_string(), e.id.into());
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

/// 泛型可扩展 Event 模型
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
    T: ToEvent<TypeLevel>,
    D: ToEvent<DetailTypeLevel>,
    S: ToEvent<SubTypeLevel>,
    P: ToEvent<PlatformLevel>,
    I: ToEvent<ImplLevel>,
{
    fn from(mut event: BaseEvent<T, D, S, P, I>) -> Self {
        Self {
            id: event.id,
            time: event.time,
            ty: event.ty.ty().to_string(),
            detail_type: event.detail_type.ty().to_string(),
            sub_type: event.sub_type.ty().to_string(),
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

pub trait ParseEvent: Sized {
    fn parse(event: Event, implt: &str) -> Result<Self, WalleError>;
}

impl<T, D, S, P, I> ParseEvent for BaseEvent<T, D, S, P, I>
where
    I: TryFromEvent<ImplLevel>,
    T: TryFromEvent<TypeLevel>,
    D: TryFromEvent<DetailTypeLevel>,
    S: TryFromEvent<SubTypeLevel>,
    P: TryFromEvent<PlatformLevel>,
{
    fn parse(mut event: Event, implt: &str) -> Result<Self, WalleError> {
        Ok(Self {
            ty: T::try_from_event_mut(&mut event, implt)?,
            detail_type: D::try_from_event_mut(&mut event, implt)?,
            sub_type: S::try_from_event_mut(&mut event, implt)?,
            implt: I::try_from_event_mut(&mut event, implt)?,
            platform: P::try_from_event_mut(&mut event, implt)?,
            id: event.id,
            time: event.time,
            extra: event.extra,
        })
    }
}

impl TryFromEvent<ImplLevel> for () {
    fn try_from_event_mut(_event: &mut Event, _implt: &str) -> WalleResult<Self> {
        Ok(())
    }
}
impl TryFromEvent<TypeLevel> for () {
    fn try_from_event_mut(_event: &mut Event, _implt: &str) -> WalleResult<Self> {
        Ok(())
    }
}
impl TryFromEvent<DetailTypeLevel> for () {
    fn try_from_event_mut(_event: &mut Event, _implt: &str) -> WalleResult<Self> {
        Ok(())
    }
}
impl TryFromEvent<SubTypeLevel> for () {
    fn try_from_event_mut(_event: &mut Event, _implt: &str) -> WalleResult<Self> {
        Ok(())
    }
}
impl TryFromEvent<PlatformLevel> for () {
    fn try_from_event_mut(_event: &mut Event, _implt: &str) -> WalleResult<Self> {
        Ok(())
    }
}
impl ToEvent<TypeLevel> for () {
    fn ty(&self) -> &'static str {
        ""
    }
}
impl ToEvent<DetailTypeLevel> for () {
    fn ty(&self) -> &'static str {
        ""
    }
}
impl ToEvent<SubTypeLevel> for () {
    fn ty(&self) -> &'static str {
        ""
    }
}
impl ToEvent<PlatformLevel> for () {
    fn ty(&self) -> &'static str {
        ""
    }
}
impl ToEvent<ImplLevel> for () {
    fn ty(&self) -> &'static str {
        ""
    }
}

impl<T, D, S, P> TryFrom<Event> for BaseEvent<T, D, S, P>
where
    T: TryFromEvent<TypeLevel>,
    D: TryFromEvent<DetailTypeLevel>,
    S: TryFromEvent<SubTypeLevel>,
    P: TryFromEvent<PlatformLevel>,
{
    type Error = WalleError;
    fn try_from(event: Event) -> Result<Self, Self::Error> {
        Self::parse(event, "")
    }
}

use walle_macro::{
    _PushToValueMap as PushToValueMap, _ToEvent as ToEvent, _TryFromEvent as TryFromEvent,
    _TryFromValue as TryFromValue,
};

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub struct Message {
    pub selft: Selft,
    pub message_id: String,
    pub message: crate::segment::Segments,
    pub alt_message: String,
    pub user_id: String,
}
pub type MessageEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Message, D, S, P, I>;

impl GetSelf for Message {
    fn get_self(&self) -> Selft {
        self.selft.clone()
    }
}

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub struct Notice {
    pub selft: Selft,
}
pub type NoticeEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Notice, D, S, P, I>;

impl GetSelf for Notice {
    fn get_self(&self) -> Selft {
        self.selft.clone()
    }
}

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub struct Request {
    pub selft: Selft,
}
pub type RequestEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Request, D, S, P, I>;

impl GetSelf for Request {
    fn get_self(&self) -> Selft {
        self.selft.clone()
    }
}

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub struct Meta;
pub type MetaEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Meta, D, S, P, I>;

#[derive(Debug, Clone, PartialEq, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub enum EventType {
    Meta,
    Message(Message),
    Request(Request),
    Notice(Notice),
}

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct Private;
pub type PrivateMessageEvent<S = (), P = (), I = ()> = BaseEvent<Message, Private, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct Group {
    pub group_id: String,
}
pub type GroupMessageEvent<S = (), P = (), I = ()> = BaseEvent<Message, Group, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct Connect {
    pub version: crate::structs::Version,
}
pub type ConnectEvent<S = (), P = (), I = ()> = BaseEvent<Meta, Connect, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct Heartbeat {
    pub interval: u32,
}
pub type HeartbeatEvent<S = (), P = (), I = ()> = BaseEvent<Meta, Heartbeat, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct StatusUpdate {
    pub status: crate::structs::Status,
}
pub type StatusUpdateEvent<S = (), P = (), I = ()> = BaseEvent<Meta, StatusUpdate, S, P, I>;

#[derive(Debug, Clone, PartialEq, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub enum MetaTypes {
    Connect(Connect),
    Heartbeat(Heartbeat),
    StatusUpdate(StatusUpdate),
}
pub type MetaDetailEvent<S = (), P = (), I = ()> = BaseEvent<Meta, MetaTypes, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GroupMemberIncrease {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GroupMemberIncreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMemberIncrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GroupMemberDecrease {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GroupMemberDecreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMemberDecrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GroupMessageDelete {
    pub group_id: String,
    pub message_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GroupMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMessageDelete, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct FriendIncrease {
    pub user_id: String,
}
pub type FriendIncreaseEvent<S = (), P = (), I = ()> = BaseEvent<Notice, FriendIncrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct FriendDecrease {
    pub user_id: String,
}
pub type FriendDecreaseEvent<S = (), P = (), I = ()> = BaseEvent<Notice, FriendDecrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct PrivateMessageDelete {
    pub message_id: String,
    pub user_id: String,
}
pub type PrivateMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, PrivateMessageDelete, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GuildMemberIncrease {
    pub guild_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GuildMemberIncreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GuildMemberIncrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GuildMemberDecrease {
    pub guild_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GuildMemberDecreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GuildMemberDecrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
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

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct ChannelCreate {
    pub guild_id: String,
    pub channel_id: String,
    pub operator_id: String,
}
pub type ChannelCreateEvent<S = (), P = (), I = ()> = BaseEvent<Notice, ChannelCreate, S, P, I>;

#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct ChannelDelete {
    pub guild_id: String,
    pub channel_id: String,
    pub operator_id: String,
}
pub type ChannelDeleteEvent<S = (), P = (), I = ()> = BaseEvent<Notice, ChannelDelete, S, P, I>;

#[derive(Debug, Clone, PartialEq, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub enum MessageDeatilTypes {
    Group(Group),
    Private(Private),
}
