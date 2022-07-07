use crate::{
    extended_value,
    prelude::WalleError,
    util::{ExtendedMap, ExtendedMapExt, ExtendedValue},
};

pub type Message = Vec<MessageSegment>;

pub struct MessageSegment {
    pub ty: String,
    pub data: ExtendedMap,
}

impl From<MessageSegment> for ExtendedValue {
    fn from(segment: MessageSegment) -> Self {
        extended_value!({
            "type": segment.ty,
            "data": segment.data
        })
    }
}

impl TryFrom<ExtendedValue> for MessageSegment {
    type Error = WalleError;
    fn try_from(value: ExtendedValue) -> Result<Self, Self::Error> {
        if let ExtendedValue::Map(mut map) = value {
            Ok(Self {
                ty: map.remove_downcast("type")?,
                data: map
                    .remove("data")
                    .ok_or(WalleError::MapMissedKey("data".to_string()))?
                    .downcast_map()?,
            })
        } else {
            Err(WalleError::ValueTypeNotMatch(
                "map".to_string(),
                format!("{:?}", value),
            ))
        }
    }
}
