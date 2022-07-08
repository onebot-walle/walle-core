use crate::util::ExtendedMapExt;
use walle_macro::{_PushToMap as PushToMap, _TryFromValue as TryFromValue};

#[derive(Debug, Clone, PartialEq, Eq, PushToMap, TryFromValue)]
pub struct Status {
    pub good: bool,
    pub online: bool,
}

// impl Into<ExtendedValue> for Status {
//     fn into(self) -> ExtendedValue {
//         extended_value!({
//             "good": self.good,
//             "data": self.online,
//         })
//     }
// }

// impl TryFrom<ExtendedValue> for Status {
//     type Error = WalleError;
//     fn try_from(value: ExtendedValue) -> Result<Self, Self::Error> {
//         if let ExtendedValue::Map(mut map) = value {
//             Ok(Self {
//                 good: map.remove_downcast("good")?,
//                 online: map.remove_downcast("online")?,
//             })
//         } else {
//             Err(WalleError::ValueTypeNotMatch(
//                 "map".to_string(),
//                 format!("{:?}", value),
//             ))
//         }
//     }
// }
