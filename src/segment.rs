use crate::{
    extended_map, extended_value,
    prelude::WalleError,
    util::{Value, ValueMap, ValueMapExt},
};
use walle_macro::{_OneBot as OneBot, _PushToValueMap as PushToValueMap};

pub type Segments = Vec<MessageSegment>;

#[derive(Debug, Clone, PartialEq)]
pub struct MessageSegment {
    pub ty: String,
    pub data: ValueMap,
}

impl ValueMapExt for MessageSegment {
    fn get_downcast<T>(&self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.data.get_downcast(key)
    }
    fn remove_downcast<T>(&mut self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.data.remove_downcast(key)
    }
    fn try_get_downcast<T>(&self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.data.try_get_downcast(key)
    }
    fn try_remove_downcast<T>(&mut self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.data.try_remove_downcast(key)
    }
    fn push<T>(&mut self, value: T)
    where
        T: crate::util::PushToValueMap,
    {
        value.push_to(&mut self.data)
    }
}

impl MessageSegment {
    pub fn alt(&self) -> String {
        if self.ty == "text" {
            self.data.get_downcast("text").unwrap_or_default()
        } else {
            if self.data.is_empty() {
                format!("[{}]", self.ty)
            } else {
                let mut content = serde_json::to_string(&self.data).unwrap_or_default();
                content.pop();
                content.remove(0);
                format!("[{},{}]", self.ty, content)
            }
        }
    }
}

impl From<MessageSegment> for Value {
    fn from(segment: MessageSegment) -> Self {
        extended_value!({
            "type": segment.ty,
            "data": segment.data
        })
    }
}

impl TryFrom<Value> for MessageSegment {
    type Error = WalleError;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        if let Value::Map(mut map) = value {
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

#[derive(Debug, Clone, PartialEq)]
pub struct BaseSegment<T> {
    pub segment: T,
    pub extra: ValueMap,
}

impl<T: for<'a> TryFrom<&'a mut MessageSegment, Error = WalleError>> TryFrom<MessageSegment>
    for BaseSegment<T>
{
    type Error = WalleError;
    fn try_from(mut segment: MessageSegment) -> Result<Self, Self::Error> {
        Ok(Self {
            segment: T::try_from(&mut segment)?,
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

impl<T: Into<MessageSegment>> IntoMessage for T {
    fn into_message(self) -> Segments {
        vec![self.into()]
    }
}

impl From<String> for MessageSegment {
    fn from(text: String) -> Self {
        MessageSegment {
            ty: "text".to_string(),
            data: extended_map! { "text": text },
        }
    }
}

impl From<&str> for MessageSegment {
    fn from(text: &str) -> Self {
        MessageSegment {
            ty: "text".to_string(),
            data: extended_map! { "text": text },
        }
    }
}

pub trait SegmentDeclare {
    fn ty() -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[segment]
pub struct Text {
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[segment]
pub struct Mention {
    pub user_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[segment]
pub struct MentionAll {}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[segment]
pub struct Image {
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[segment]
pub struct Voice {
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[segment]
pub struct Audio {
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[segment]
pub struct Video {
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[segment]
pub struct File {
    pub file_id: String,
}

#[derive(Debug, Clone, PartialEq, PushToValueMap, OneBot)]
#[segment]
pub struct Location {
    pub latitude: f64,
    pub longitude: f64,
    pub title: String,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PushToValueMap, OneBot)]
#[segment]
pub struct Reply {
    pub message_id: String,
    pub user_id: String,
}

pub trait MessageExt {
    fn extract_plain_text(&self) -> String;
    fn extract<T: TryFrom<MessageSegment>>(self) -> Vec<T>;
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
    fn extract<T: TryFrom<MessageSegment>>(self) -> Vec<T> {
        self.into_iter()
            .filter_map(|seg| T::try_from(seg).ok())
            .collect()
    }
}
