//! # Event 模型定义
//! 定义了以下几个类型用于 OneBot 项目的日常使用
//!
//! - `Event`：标准的 Event 模型，确保事件最基本的字段，所有其他字段可以后续再继续处理，可以序列化和反序列化。
//! - `BaseEvent<T, D, S, P, I>`：根据 Rust 类型系统设计的可扩展模型，使用五个层级的泛型分别持有五个层级的扩展字段，
//! 可以尝试从 `Event` 转化，或转化到 `Event`，不可直接序列化和反序列化，可以用于更好的在实现端构建事件以及在应用端处理事件。

use crate::{
    prelude::{WalleError, WalleResult},
    structs::Selft,
    util::{GetSelf, PushToValueMap, Value, ValueMap, ValueMapExt},
};

use serde::{Deserialize, Serialize};

/// 标准 Event 模型
///
/// 用字符串储存五个扩展层级的字段，并包含标准规定的必须持有的 `id` 和 `time` 字段，`extra` 持有所有其他字段
///
/// 可以直接用于序列化和反序列化
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

/// 所有 `BaseEvent` 的泛型都需要实现该 trait 才能转化为 `Event`
///
/// 泛型 `T` 可以取 `TypeLevel`/`DetailTypeLevel`/`SubTypeLevel`/`PlatformLevel`/`ImplLevel` 五个标记等级
///
/// 通常情况可以通过 `TryFromEvent` derive 宏实现，但也可以手动实现，例如：
///
/// ```rust
/// use walle_core::prelude::*;
///
/// #[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
/// #[event(type)] // 标记扩展等级为 type
/// pub struct Message {
///     pub selft: Selft,
///     pub message_id: String,
///     pub message: Segments,
///     pub alt_message: String,
///     pub user_id: String,
/// }
/// ```
pub trait ToEvent<T>: PushToValueMap {
    /// 返回为该扩展等级的标记字符串
    fn ty(&self) -> &'static str;
}

/// 所有 `BaseEvent` 的泛型都需要实现该 trait 才能尝试从 `Event` 转化而来
///
/// 泛型 `T` 可以取 `TypeLevel`/`DetailTypeLevel`/`SubTypeLevel`/`PlatformLevel`/`ImplLevel` 五个标记等级
///
/// 通常情况可以通过 `TryFromEvent` derive 宏实现，但也可以手动实现，例如：
///
/// ```rust
/// use walle_core::prelude::*;
///
/// #[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
/// #[event(type)] // 标记扩展等级为 type
/// pub struct Message {
///     pub selft: Selft,
///     pub message_id: String,
///     pub message: Segments,
///     pub alt_message: String,
///     pub user_id: String,
/// }
/// ```
pub trait TryFromEvent<T>: Sized {
    fn try_from_event_mut(event: &mut Event, implt: &str) -> WalleResult<Self>;
    fn try_from_event(mut event: Event, implt: &str) -> WalleResult<Self> {
        Self::try_from_event_mut(&mut event, implt)
    }
}

/// 仅用于标记，标记扩展等级为 type 的 trait
pub struct TypeLevel;
/// 仅用于标记，标记扩展等级为 detail_type 的 trait
pub struct DetailTypeLevel;
/// 仅用于标记，标记扩展等级为 sub_type 的 trait
pub struct SubTypeLevel;
/// 仅用于标记，标记扩展等级为 platform 的 trait
pub struct PlatformLevel;
/// 仅用于标记，标记扩展等级为 impl 的 trait
pub struct ImplLevel;

impl Event {
    /// 尝试获取 `self` 字段
    pub fn selft(&self) -> Option<Selft> {
        self.extra.get_downcast("self").ok()
    }
    /// 尝试获取 `self` 字段中的 `user_id` 字段
    pub fn self_id(&self) -> Option<String> {
        self.selft().map(|s| s.user_id)
    }
    /// 尝试获取 `self` 字段中的 `platform` 字段
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
///
/// 用于在实现端构建事件以及在应用端处理事件，可以尝试从 `Event` 转化，或转化到 `Event`，不可直接序列化和反序列化
///
/// 只有当满足以下所有条件时，该BaseEvent 才可以尝试从 `Event` 转化：
/// - `T` 实现 `TryFromEvent<TypeLevel>`
/// - `D` 实现 `TryFromEvent<DetailTypeLevel>`
/// - `S` 实现 `TryFromEvent<SubTypeLevel>`
/// - `P` 实现 `TryFromEvent<PlatformLevel>`
/// - `I` 实现 `TryFromEvent<ImplLevel>`
///
/// 只有当满足以下所有条件时，该BaseEvent 才可以转化到 `Event`：
/// - `T` 实现 `ToEvent<TypeLevel>`
/// - `D` 实现 `ToEvent<DetailTypeLevel>`
/// - `S` 实现 `ToEvent<SubTypeLevel>`
/// - `P` 实现 `ToEvent<PlatformLevel>`
/// - `I` 实现 `ToEvent<ImplLevel>`
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

impl<T, D, S, P, I> TryFrom<(Event, &str)> for BaseEvent<T, D, S, P, I>
where
    T: TryFromEvent<TypeLevel>,
    D: TryFromEvent<DetailTypeLevel>,
    S: TryFromEvent<SubTypeLevel>,
    P: TryFromEvent<PlatformLevel>,
    I: TryFromEvent<ImplLevel>,
{
    type Error = WalleError;
    fn try_from((mut event, implt): (Event, &str)) -> Result<Self, Self::Error> {
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

/// OneBot 12 标准事件 `message` 字段结构体
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub struct Message {
    pub selft: Selft,
    pub message_id: String,
    pub message: crate::segment::Segments,
    pub alt_message: String,
    pub user_id: String,
}
/// OneBot 12 `message` 事件
pub type MessageEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Message, D, S, P, I>;

impl GetSelf for Message {
    fn get_self(&self) -> Selft {
        self.selft.clone()
    }
}

/// OneBot 12 标准事件 `notice` 字段结构体
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub struct Notice {
    pub selft: Selft,
}
/// OneBot 12 `notice` 事件
pub type NoticeEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Notice, D, S, P, I>;

impl GetSelf for Notice {
    fn get_self(&self) -> Selft {
        self.selft.clone()
    }
}

/// OneBot 12 标准事件 `request` 字段结构体
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub struct Request {
    pub selft: Selft,
}
/// OneBot 12 `request` 事件
pub type RequestEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Request, D, S, P, I>;

impl GetSelf for Request {
    fn get_self(&self) -> Selft {
        self.selft.clone()
    }
}

/// OneBot 12 标准事件 `meta` 字段结构体（其实啥字段也没有）
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub struct Meta;
/// OneBot 12 `meta` 事件
pub type MetaEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Meta, D, S, P, I>;

/// OneBot 12 标准事件 `type` 层级所有可能值枚举
#[derive(Debug, Clone, PartialEq, PushToValueMap, ToEvent, TryFromEvent)]
#[event(type)]
pub enum EventType {
    Meta,
    Message(Message),
    Request(Request),
    Notice(Notice),
}

/// OneBot 12 标准事件 `detail_type` 层级 `private` 字段结构体
///
/// 用于 `message.private`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct Private;
/// OneBot 12 `message.private` 事件
pub type PrivateMessageEvent<S = (), P = (), I = ()> = BaseEvent<Message, Private, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `group` 字段结构体
///
/// 用于 `message.group`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct Group {
    pub group_id: String,
}
/// OneBot 12 `message.group` 事件
pub type GroupMessageEvent<S = (), P = (), I = ()> = BaseEvent<Message, Group, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `channel` 字段结构体
///
/// 用于 `message.channel`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct Channel {
    pub guild_id: String,
    pub channel_id: String,
}
/// OneBot 12 `message.channel` 事件
pub type ChannelMessageEvent<S = (), P = (), I = ()> = BaseEvent<Message, Channel, S, P, I>;

/// OneBot 12 标准 `message` 事件 `detail_type` 层级所有可能值枚举
#[derive(Debug, Clone, PartialEq, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub enum MessageDetailTypes {
    Group(Group),
    Private(Private),
    Channel(Channel),
}
/// OneBot 12 `message` 所有事件枚举
pub type MessageDetailEvent<S = (), P = (), I = ()> =
    BaseEvent<Message, MessageDetailTypes, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `connect` 字段结构体
///
/// 用于 `meta.connect`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct Connect {
    pub version: crate::structs::Version,
}
/// OneBot 12 `meta.connect` 事件
pub type ConnectEvent<S = (), P = (), I = ()> = BaseEvent<Meta, Connect, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `heartbeat` 字段结构体
///
/// 用于 `meta.heartbeat`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct Heartbeat {
    pub interval: u32,
}
/// OneBot 12 `meta.heartbeat` 事件
pub type HeartbeatEvent<S = (), P = (), I = ()> = BaseEvent<Meta, Heartbeat, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `status_update` 字段结构体
///
/// 用于 `meta.status_update`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct StatusUpdate {
    pub status: crate::structs::Status,
}
/// OneBot 12 `meta.status_update` 事件
pub type StatusUpdateEvent<S = (), P = (), I = ()> = BaseEvent<Meta, StatusUpdate, S, P, I>;

/// OneBot 12 标准 `meta` 事件 `detail_type` 层级所有可能值枚举
#[derive(Debug, Clone, PartialEq, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub enum MetaTypes {
    Connect(Connect),
    Heartbeat(Heartbeat),
    StatusUpdate(StatusUpdate),
}
/// OneBot 12 `meta` 所有事件枚举
pub type MetaDetailEvent<S = (), P = (), I = ()> = BaseEvent<Meta, MetaTypes, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `group_member_increase` 字段结构体
///
/// 用于 `notice.group_member_increase`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GroupMemberIncrease {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}
/// OneBot 12 `notice.group_member_increase` 事件
pub type GroupMemberIncreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMemberIncrease, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `group_member_decrease` 字段结构体
///
/// 用于 `notice.group_member_decrease`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GroupMemberDecrease {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}
/// OneBot 12 `notice.group_member_decrease` 事件
pub type GroupMemberDecreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMemberDecrease, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `group_message_delete` 字段结构体
///
/// 用于 `notice.group_message_delete`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GroupMessageDelete {
    pub group_id: String,
    pub message_id: String,
    pub user_id: String,
    pub operator_id: String,
}
/// OneBot 12 `notice.group_message_delete` 事件
pub type GroupMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMessageDelete, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `friend_increase` 字段结构体
///
/// 用于 `notice.friend_increase`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct FriendIncrease {
    pub user_id: String,
}
/// OneBot 12 `notice.friend_increase` 事件
pub type FriendIncreaseEvent<S = (), P = (), I = ()> = BaseEvent<Notice, FriendIncrease, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `friend_decrease` 字段结构体
///
/// 用于 `notice.friend_decrease`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct FriendDecrease {
    pub user_id: String,
}
/// OneBot 12 `notice.friend_decrease` 事件
pub type FriendDecreaseEvent<S = (), P = (), I = ()> = BaseEvent<Notice, FriendDecrease, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `private_message_delete` 字段结构体
///
/// 用于 `notice.private_message_delete`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct PrivateMessageDelete {
    pub message_id: String,
    pub user_id: String,
}
/// OneBot 12 `notice.private_message_delete` 事件
pub type PrivateMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, PrivateMessageDelete, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `guild_member_increase` 字段结构体
///
/// 用于 `notice.guild_member_increase`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GuildMemberIncrease {
    pub guild_id: String,
    pub user_id: String,
    pub operator_id: String,
}
/// OneBot 12 `notice.guild_member_increase` 事件
pub type GuildMemberIncreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GuildMemberIncrease, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `guild_member_decrease` 字段结构体
///
/// 用于 `notice.guild_member_decrease`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct GuildMemberDecrease {
    pub guild_id: String,
    pub user_id: String,
    pub operator_id: String,
}
/// OneBot 12 `notice.guild_member_decrease` 事件
pub type GuildMemberDecreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GuildMemberDecrease, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `channel_message_delete` 字段结构体
///
/// 用于 `notice.channel_message_delete`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct ChannelMessageDelete {
    pub guild_id: String,
    pub channel_id: String,
    pub user_id: String,
    pub operator_id: String,
    pub message_id: String,
}
/// OneBot 12 `notice.channel_message_delete` 事件
pub type ChannelMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, ChannelMessageDelete, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `channel_create` 字段结构体
///
/// 用于 `notice.channel_create`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct ChannelCreate {
    pub guild_id: String,
    pub channel_id: String,
    pub operator_id: String,
}
/// OneBot 12 `notice.channel_create` 事件
pub type ChannelCreateEvent<S = (), P = (), I = ()> = BaseEvent<Notice, ChannelCreate, S, P, I>;

/// OneBot 12 标准事件 `detail_type` 层级 `channel_delete` 字段结构体
///
/// 用于 `notice.channel_delete`
#[derive(Debug, Clone, PartialEq, TryFromValue, PushToValueMap, ToEvent, TryFromEvent)]
#[event(detail_type)]
pub struct ChannelDelete {
    pub guild_id: String,
    pub channel_id: String,
    pub operator_id: String,
}
/// OneBot 12 `notice.channel_delete` 事件
pub type ChannelDeleteEvent<S = (), P = (), I = ()> = BaseEvent<Notice, ChannelDelete, S, P, I>;
