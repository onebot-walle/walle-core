use crate::{
    prelude::WalleError,
    util::{ExtendedMap, PushToExtendedMap, SelfId},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    #[serde(rename = "impl")]
    pub implt: String,
    pub platform: String,
    pub self_id: String,
    pub time: f64,
    #[serde(rename = "type")]
    pub ty: String,
    pub detail_type: String,
    pub sub_type: String,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

impl SelfId for Event {
    fn self_id(&self) -> String {
        self.self_id.to_string()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseEvent<T = (), D = (), S = (), P = (), I = ()> {
    pub id: String,
    pub self_id: String,
    pub time: f64,
    pub implt: I,
    pub platform: P,
    pub ty: T,
    pub detail_type: D,
    pub sub_type: S,
    pub extra: ExtendedMap,
}

impl<T, D, S, P, I> From<BaseEvent<T, D, S, P, I>> for Event
where
    T: TypeDeclare + PushToExtendedMap,
    D: DetailTypeDeclare + PushToExtendedMap,
    S: SubTypeDeclare + PushToExtendedMap,
    P: PlatformDeclare + PushToExtendedMap,
    I: ImplDeclare + PushToExtendedMap,
{
    fn from(mut event: BaseEvent<T, D, S, P, I>) -> Self {
        Self {
            id: event.id,
            implt: I::implt().to_string(),
            platform: P::platform().to_string(),
            self_id: event.self_id,
            time: event.time,
            ty: T::ty().to_string(),
            detail_type: D::detail_type().to_string(),
            sub_type: S::sub_type().to_string(),
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
    self_id: String,
    ty: T,
    detail_type: D,
    sub_type: S,
    platform: P,
    implt: I,
    extra: ExtendedMap,
) -> BaseEvent<T, D, S, P, I> {
    BaseEvent::<T, D, S, P, I> {
        id,
        time,
        self_id,
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
    fn try_from(mut value: Event) -> Result<Self, Self::Error> {
        let event = &mut value;
        Ok(Self {
            ty: T::try_from(event)?,
            detail_type: D::try_from(event)?,
            sub_type: S::try_from(event)?,
            implt: I::try_from(event)?,
            platform: P::try_from(event)?,
            id: value.id,
            self_id: value.self_id,
            time: value.time,
            extra: value.extra,
        })
    }
}

pub trait ImplDeclare {
    fn implt() -> &'static str {
        ""
    }
}

pub trait PlatformDeclare {
    fn platform() -> &'static str {
        ""
    }
}

pub trait SubTypeDeclare {
    fn sub_type() -> &'static str {
        ""
    }
}

pub trait DetailTypeDeclare {
    fn detail_type() -> &'static str {
        ""
    }
}

pub trait TypeDeclare {
    fn ty() -> &'static str {
        ""
    }
}

impl TypeDeclare for () {}
impl DetailTypeDeclare for () {}
impl SubTypeDeclare for () {}
impl PlatformDeclare for () {}
impl ImplDeclare for () {}
impl PushToExtendedMap for () {}
impl TryFrom<&mut Event> for () {
    type Error = WalleError;
    fn try_from(_: &mut Event) -> Result<Self, Self::Error> {
        Ok(())
    }
}

// pub struct MessageE {
//     pub message_id: String,
//     pub message: crate::message_next::Message,
//     pub alt_message: String,
//     pub user_id: String,
// }

// impl TypeDeclare for MessageE {
//     fn ty() -> &'static str {
//         "message"
//     }
// }

// impl TryFrom<&mut Event> for MessageE {
//     type Error = WalleError;
//     fn try_from(value: &mut Event) -> Result<Self, Self::Error> {
//         if value.ty == Self::ty() {
//             Ok(Self {
//                 message_id: value.extra.remove_downcast("message_id")?,
//                 message: value.extra.remove_downcast("message")?,
//                 alt_message: value.extra.remove_downcast("alt_message")?,
//                 user_id: value.extra.remove_downcast("user_id")?,
//             })
//         } else {
//             Err(WalleError::EventDeclareNotMatch(
//                 Self::ty(),
//                 value.ty.clone(),
//             ))
//         }
//     }
// }

// impl PushToExtendedMap for MessageE {
//     fn push(self, map: &mut ExtendedMap) {
//         map.insert("message_id".to_string(), self.message_id.into());
//         map.insert("message".to_string(), self.message.into());
//         map.insert("alt_message".to_string(), self.alt_message.into());
//         map.insert("user_id".to_string(), self.user_id.into());
//     }
// }

// impl Into<ExtendedValue> for MessageE {
//     fn into(self) -> ExtendedValue {
//         let mut map = ExtendedMap::default();
//         self.push(&mut map);
//         ExtendedValue::Map(map)
//     }
// }

use walle_macro::{_OneBot as OneBot, _PushToMap as PushToMap};

#[derive(Debug, Clone, PartialEq, OneBot, PushToMap)]
#[event(type)]
pub struct Message {
    pub message_id: String,
    pub message: crate::message_next::Message,
    pub alt_message: String,
    pub user_id: String,
}
pub type MessageEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Message, D, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(type)]
pub struct Notice {}
pub type NoticeEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Notice, D, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(type)]
pub struct Request {}
pub type RequestEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Request, D, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(type)]
pub struct Meta {}
pub type MetaEvent<D = (), S = (), P = (), I = ()> = BaseEvent<Meta, D, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct Private {}
pub type PrivateMessageEvent<S = (), P = (), I = ()> = BaseEvent<Message, Private, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct Group {
    pub group_id: String,
}
pub type GroupMessageEvent<S = (), P = (), I = ()> = BaseEvent<Message, Group, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct Heartbeat {
    pub interval: u32,
    pub status: crate::structs::Status,
}
pub type HeartbeatEvent<S = (), P = (), I = ()> = BaseEvent<Meta, Heartbeat, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct GroupMemberIncrease {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GroupMemberIncreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMemberIncrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct GroupMemberDecrease {
    pub group_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GroupMemberDecreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMemberDecrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct GroupMessageDelete {
    pub group_id: String,
    pub message_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GroupMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GroupMessageDelete, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct FriendIncrease {
    pub user_id: String,
}
pub type FriendIncreaseEvent<S = (), P = (), I = ()> = BaseEvent<Notice, FriendIncrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct FriendDecrease {
    pub user_id: String,
}
pub type FriendDecreaseEvent<S = (), P = (), I = ()> = BaseEvent<Notice, FriendDecrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct PrivateMessageDelete {
    pub message_id: String,
    pub user_id: String,
}
pub type PrivateMessageDeleteEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, PrivateMessageDelete, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct GuildMemberIncrease {
    pub guild_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GuildMemberIncreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GuildMemberIncrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct GuildMemberDecrease {
    pub guild_id: String,
    pub user_id: String,
    pub operator_id: String,
}
pub type GuildMemberDecreaseEvent<S = (), P = (), I = ()> =
    BaseEvent<Notice, GuildMemberDecrease, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
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

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct ChannelCreate {
    pub guild_id: String,
    pub channel_id: String,
    pub operator_id: String,
}
pub type ChannelCreateEvent<S = (), P = (), I = ()> = BaseEvent<Notice, ChannelCreate, S, P, I>;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[event(detail_type)]
pub struct ChannelDelete {
    pub guild_id: String,
    pub channel_id: String,
    pub operator_id: String,
}
pub type ChannelDeleteEvent<S = (), P = (), I = ()> = BaseEvent<Notice, ChannelDelete, S, P, I>;
