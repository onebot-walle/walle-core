use crate::{
    event::{ToEvent, TypeLevel},
    util::PushToValueMap,
};

pub struct EventTypeNamedStruct {
    pub struct_field0: String,
}

impl ToEvent<TypeLevel> for EventTypeNamedStruct {
    fn ty(&self) -> &'static str {
        ""
    }
}

impl PushToValueMap for EventTypeNamedStruct {
    fn push_to(self, map: &mut crate::util::ValueMap) {
        map.insert("struct_field0".to_string(), self.struct_field0.into());
    }
}

use walle_macro::{
    _PushToValueMap as PushToValueMap, _ToEvent as ToEvent, _TryFromEvent as TryFromEvent,
};

#[derive(ToEvent, PushToValueMap, TryFromEvent)]
#[event(type = "type", detail_type = "detail_type")]
pub enum EventType {
    A { f: u16 },
    B,
}
