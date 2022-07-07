use crate::{
    prelude::WalleError,
    util::{ExtendedMap, PushToExtendedMap},
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

#[derive(Debug, Clone, PartialEq)]
pub struct BaseEvent<T = (), D = (), S = (), P = (), I = ()>
where
    T: TypeDeclare,
    D: DetailTypeDeclare,
    S: SubTypeDeclare,
    P: PlatformDeclare,
    I: ImplDeclare,
{
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
                event.implt.push(map);
                event.platform.push(map);
                event.ty.push(map);
                event.detail_type.push(map);
                event.sub_type.push(map);
                event.extra
            },
        }
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

use walle_macro::EventContent;

#[derive(Debug, Clone, PartialEq, EventContent)]
#[event(type = "message")]
#[internal]
pub struct Message {
    pub message_id: String,
    pub message: crate::message_next::Message,
    pub alt_message: String,
    pub user_id: String,
}
pub type MessageEvent = BaseEvent<Message>;

#[derive(Debug, Clone, PartialEq, Eq, EventContent)]
#[event(type = "notice")]
#[internal]
pub struct Notice {}
pub type NoticeEvent = BaseEvent<Notice>;

#[derive(Debug, Clone, PartialEq, Eq, EventContent)]
#[event(type = "request")]
#[internal]
pub struct Request {}
pub type RequestEvent = BaseEvent<Request>;

#[derive(Debug, Clone, PartialEq, Eq, EventContent)]
#[event(type = "meta")]
#[internal]
pub struct Meta {}
pub type MetaEvent = BaseEvent<Meta>;

#[derive(Debug, Clone, PartialEq, Eq, EventContent)]
#[event(detail_type = "private")]
#[internal]
pub struct Private {}
pub type PrivateMessageEvent = BaseEvent<Message, Private>;

#[derive(Debug, Clone, PartialEq, Eq, EventContent)]
#[event(detail_type = "group")]
#[internal]
pub struct Group {
    pub group_id: String,
}
pub type GroupMessageEvent = BaseEvent<Message, Group>;
