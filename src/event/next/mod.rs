use crate::{
    prelude::WalleError,
    util::{ExtendedMap, ExtendedMapExt, PushToExtendedMap},
};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Event {
    pub id: String,
    #[serde(rename = "impl")]
    pub r#impl: String,
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
    pub r#impl: I,
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
            r#impl: I::r#impl().to_string(),
            platform: P::platform().to_string(),
            self_id: event.self_id,
            time: event.time,
            ty: T::ty().to_string(),
            detail_type: D::detail_type().to_string(),
            sub_type: S::sub_type().to_string(),
            extra: {
                let map = &mut event.extra;
                event.r#impl.push(map);
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
            r#impl: I::try_from(event)?,
            platform: P::try_from(event)?,
            id: value.id,
            self_id: value.self_id,
            time: value.time,
            extra: value.extra,
        })
    }
}

pub trait ImplDeclare {
    fn r#impl() -> &'static str {
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

macro_rules! declare_event {
    ($main: ty, r#impl $(:$d: expr)?) => {
        impl ImplDeclare for $main {
            $(fn r#impl() -> &'static str {
                $d
            })?
        }
    };
    ($main: ty, platform $(:$d: expr)?) => {
        impl PlatformDeclare for $main {
            $(fn platform() -> &'static str {
                $d
            })?
        }
    };
    ($main: ty, sub_type $(:$d: expr)?) => {
        impl SubTypeDeclare for $main {
            $(fn sub_type() -> &'static str {
                $d
            })?
        }
    };
    ($main: ty, detail_type $(:$d: expr)?) => {
        impl DetailTypeDeclare for $main {
            $(fn detail_type() -> &'static str {
                $d
            })?
        }
    };
    ($main: ty, ty $(:$d: expr)?) => {
        impl TypeDeclare for $main {
            $(fn ty() -> &'static str {
                $d
            })?
        }
    };
}

declare_event!((), r#impl);
declare_event!((), platform);
declare_event!((), sub_type);
declare_event!((), detail_type);
declare_event!((), ty);
impl PushToExtendedMap for () {}
impl TryFrom<&mut Event> for () {
    type Error = WalleError;
    fn try_from(_: &mut Event) -> Result<Self, Self::Error> {
        Ok(())
    }
}

pub struct MessageE {
    pub message_id: String,
    pub message: crate::message_next::Message,
    pub alt_message: String,
    pub user_id: String,
}

declare_event!(MessageE, detail_type: "message");

impl TryFrom<&mut Event> for MessageE {
    type Error = WalleError;
    fn try_from(value: &mut Event) -> Result<Self, Self::Error> {
        if value.detail_type == Self::detail_type() {
            Ok(Self {
                message_id: value.extra.remove_downcast("message_id")?,
                message: value.extra.remove_downcast("message")?,
                alt_message: value.extra.remove_downcast("alt_message")?,
                user_id: value.extra.remove_downcast("user_id")?,
            })
        } else {
            Err(WalleError::EventDeclareNotMatch(
                Self::detail_type(),
                value.detail_type.clone(),
            ))
        }
    }
}

impl PushToExtendedMap for MessageE {
    fn push(self, map: &mut ExtendedMap) {
        map.insert("message_id".to_string(), self.message_id.into());
        map.insert("message".to_string(), self.message.into());
        map.insert("alt_message".to_string(), self.alt_message.into());
        map.insert("user_id".to_string(), self.user_id.into());
    }
}
