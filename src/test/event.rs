use crate::{
    event::{Event, TypeDeclare},
    prelude::WalleError,
    util::ValueMapExt,
};

pub struct EventTypeStruct {
    pub struct_field0: String,
}

impl TypeDeclare for EventTypeStruct {
    fn ty(&self) -> &'static str {
        "type"
    }
    fn check(event: &Event) -> bool {
        // remove
        event.ty == "type"
    }
}

impl TryFrom<&Event> for EventTypeStruct {
    type Error = WalleError;
    fn try_from(event: &Event) -> Result<Self, Self::Error> {
        if event.ty == "type" {
            Ok(Self {
                struct_field0: event.get_downcast("field0")?,
            })
        } else {
            Err(WalleError::DeclareNotMatch("type", event.ty.clone()))
        }
    }
}

impl TryFrom<&mut Event> for EventTypeStruct {
    type Error = WalleError;
    fn try_from(event: &mut Event) -> Result<Self, Self::Error> {
        if event.ty == "type" {
            Ok(Self {
                struct_field0: event.remove_downcast("field0")?,
            })
        } else {
            Err(WalleError::DeclareNotMatch("type", event.ty.clone()))
        }
    }
}
