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
