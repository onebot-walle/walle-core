use crate::{
    prelude::{WalleError, WalleResult},
    util::{PushToValueMap, Value, ValueMap, ValueMapExt},
    value, value_map,
};

pub type Segments = Vec<MsgSegment>;

#[derive(Debug, Clone, PartialEq)]
pub struct MsgSegment {
    pub ty: String,
    pub data: ValueMap,
}

pub trait ToMsgSegment: PushToValueMap {
    fn ty(&self) -> &'static str;
    fn to_segment(self) -> MsgSegment
    where
        Self: Sized,
    {
        MsgSegment {
            ty: self.ty().to_string(),
            data: {
                let mut map = ValueMap::new();
                self.push_to(&mut map);
                map
            },
        }
    }
}

pub trait TryFromMsgSegment: Sized {
    fn try_from_msg_segment_mut(segment: &mut MsgSegment) -> WalleResult<Self>;
    fn try_from_msg_segment(mut segment: MsgSegment) -> WalleResult<Self> {
        Self::try_from_msg_segment_mut(&mut segment)
    }
}

impl MsgSegment {
    pub fn alt(&self) -> String {
        if self.ty == "text" {
            self.data.get_downcast("text").unwrap_or_default()
        } else if self.data.is_empty() {
            format!("[{}]", self.ty)
        } else {
            let mut content = serde_json::to_string(&self.data).unwrap_or_default();
            content.pop();
            content.remove(0);
            format!("[{},{}]", self.ty, content)
        }
    }
}

pub fn alt(segments: &Segments) -> String {
    segments.iter().map(|seg| seg.alt()).collect()
}

impl From<MsgSegment> for Value {
    fn from(segment: MsgSegment) -> Self {
        value!({
            "type": segment.ty,
            "data": segment.data
        })
    }
}

impl TryFrom<Value> for MsgSegment {
    type Error = WalleError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Map(mut map) = value {
            Ok(Self {
                ty: map.remove_downcast("type")?,
                data: map
                    .remove("data")
                    .ok_or_else(|| WalleError::MapMissedKey("data".to_string()))?
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

#[derive(Debug, Clone, PartialEq)]
pub struct BaseSegment<T> {
    pub segment: T,
    pub extra: ValueMap,
}

impl<T> TryFrom<MsgSegment> for BaseSegment<T>
where
    T: TryFromMsgSegment,
{
    type Error = WalleError;
    fn try_from(mut segment: MsgSegment) -> Result<Self, Self::Error> {
        Ok(Self {
            segment: T::try_from_msg_segment_mut(&mut segment)?,
            extra: segment.data,
        })
    }
}

pub trait IntoMessage {
    fn into_message(self) -> Segments;
}

impl IntoMessage for Segments {
    fn into_message(self) -> Segments {
        self
    }
}

impl<T: Into<MsgSegment>> IntoMessage for T {
    fn into_message(self) -> Segments {
        vec![self.into()]
    }
}

impl From<String> for MsgSegment {
    fn from(text: String) -> Self {
        MsgSegment {
            ty: "text".to_string(),
            data: value_map! { "text": text },
        }
    }
}

impl From<&str> for MsgSegment {
    fn from(text: &str) -> Self {
        MsgSegment {
            ty: "text".to_string(),
            data: value_map! { "text": text },
        }
    }
}

pub trait SegmentDeclare {
    fn ty(&self) -> &'static str;
    fn check(segment: &MsgSegment) -> bool;
}

use walle_macro::{
    _PushToValueMap as PushToValueMap, _ToMsgSegment as ToMsgSegment,
    _TryFromMsgSegment as TryFromMsgSegment, _TryFromValue as TryFromValue,
};

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct Text {
    pub text: String,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct Mention {
    pub user_id: String,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct MentionAll {}

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct Image {
    pub file_id: String,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct Voice {
    pub file_id: String,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct Audio {
    pub file_id: String,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct Video {
    pub file_id: String,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct File {
    pub file_id: String,
}

#[derive(
    Debug, Clone, PartialEq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub title: String,
    pub content: String,
}

#[derive(
    Debug, Clone, PartialEq, Eq, PushToValueMap, ToMsgSegment, TryFromMsgSegment, TryFromValue,
)]
pub struct Reply {
    pub message_id: String,
    pub user_id: String,
}

pub trait MessageExt {
    fn extract_plain_text(&self) -> String;
    fn extract<T: TryFrom<MsgSegment>>(self) -> Vec<T>;
}

impl MessageExt for Segments {
    fn extract_plain_text(&self) -> String {
        self.iter()
            .filter_map(|segment| {
                if segment.ty == "text" {
                    Some(segment.alt())
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
    fn extract<T: TryFrom<MsgSegment>>(self) -> Vec<T> {
        self.into_iter()
            .filter_map(|seg| T::try_from(seg).ok())
            .collect()
    }
}
