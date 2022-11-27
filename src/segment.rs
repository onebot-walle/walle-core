//! MsgSegment 相关模型定义

use crate::{
    prelude::{WalleError, WalleResult},
    util::{PushToValueMap, TryAsMut, TryAsRef, Value, ValueMap, ValueMapExt},
    value, value_map,
};

pub type Segments = Vec<MsgSegment>;

/// 标准 MsgSegment 模型
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
    pub fn try_as_ref<'a>(&'a self) -> WalleResult<MsgSegmentRef<'a>> {
        self._try_as_ref()
    }
    pub fn try_as_mut<'a>(&'a mut self) -> WalleResult<MsgSegmentMut<'a>> {
        self._try_as_mut()
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

/// 泛型可扩展 MsgSegment 模型
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
    fn try_as_ref<'a>(&'a self) -> WalleResult<Vec<MsgSegmentRef<'a>>>;
    fn try_as_mut<'a>(&'a mut self) -> WalleResult<Vec<MsgSegmentMut<'a>>>;
    fn try_iter_text_mut<'a>(&'a mut self) -> WalleResult<Vec<&'a mut String>> {
        Ok(self
            .try_as_mut()?
            .into_iter()
            .filter_map(|seg| {
                if let MsgSegmentMut::Text { text } = seg {
                    Some(text)
                } else {
                    None
                }
            })
            .collect())
    }
    fn try_first_text_mut<'a>(&'a mut self) -> WalleResult<&'a mut String> {
        let mut segs = self.try_as_mut()?;
        if !segs.is_empty() {
            if let MsgSegmentMut::Text { text } = segs.remove(0) {
                return Ok(text);
            }
        }
        Err(WalleError::Other(
            "first message segment is not text".to_string(),
        ))
    }
    fn try_last_text_mut<'a>(&'a mut self) -> WalleResult<&'a mut String> {
        if let Some(MsgSegmentMut::Text { text }) = self.try_as_mut()?.pop() {
            return Ok(text);
        } else {
            Err(WalleError::Other(
                "first message segment is not text".to_string(),
            ))
        }
    }
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
    fn try_as_ref<'a>(&'a self) -> WalleResult<Vec<MsgSegmentRef<'a>>> {
        self.iter().map(|seg| seg.try_as_ref()).collect()
    }
    fn try_as_mut<'a>(&'a mut self) -> WalleResult<Vec<MsgSegmentMut<'a>>> {
        self.into_iter().map(|seg| seg.try_as_mut()).collect()
    }
}

pub enum MsgSegmentRef<'a> {
    Text {
        text: &'a str,
        extra: &'a ValueMap,
    },
    Mention {
        user_id: &'a str,
        extra: &'a ValueMap,
    },
    MentionAll {
        extra: &'a ValueMap,
    },
    Image {
        file_id: &'a str,
        extra: &'a ValueMap,
    },
    Voice {
        file_id: &'a str,
        extra: &'a ValueMap,
    },
    Audio {
        file_id: &'a str,
        extra: &'a ValueMap,
    },
    Video {
        file_id: &'a str,
        extra: &'a ValueMap,
    },
    File {
        file_id: &'a str,
        extra: &'a ValueMap,
    },
    Location {
        latitude: &'a f64,
        longitude: &'a f64,
        title: &'a str,
        content: &'a str,
        extra: &'a ValueMap,
    },
    Reply {
        message_id: &'a str,
        user_id: &'a str,
        extra: &'a ValueMap,
    },
    Other {
        ty: &'a str,
        extra: &'a ValueMap,
    },
}

fn _as_ref<'a, 'b, 'c>(ty: &'a str, data: &'b ValueMap) -> WalleResult<MsgSegmentRef<'c>>
where
    'a: 'c,
    'b: 'c,
{
    match ty {
        "text" => Ok(MsgSegmentRef::Text {
            text: data.try_get_as_ref("text")?,
            extra: &data,
        }),
        "mention" => Ok(MsgSegmentRef::Mention {
            user_id: data.try_get_as_ref("user_id")?,
            extra: &data,
        }),
        "mention_all" => Ok(MsgSegmentRef::MentionAll { extra: &data }),
        "image" => Ok(MsgSegmentRef::Image {
            file_id: data.try_get_as_ref("file_id")?,
            extra: &data,
        }),
        "voice" => Ok(MsgSegmentRef::Voice {
            file_id: data.try_get_as_ref("file_id")?,
            extra: &data,
        }),
        "audio" => Ok(MsgSegmentRef::Audio {
            file_id: data.try_get_as_ref("file_id")?,
            extra: &data,
        }),
        "video" => Ok(MsgSegmentRef::Video {
            file_id: data.try_get_as_ref("file_id")?,
            extra: &data,
        }),
        "file" => Ok(MsgSegmentRef::File {
            file_id: data.try_get_as_ref("file_id")?,
            extra: &data,
        }),
        "location" => Ok(MsgSegmentRef::Location {
            latitude: data.try_get_as_ref("latitude")?,
            longitude: data.try_get_as_ref("longitude")?,
            title: data.try_get_as_ref("title")?,
            content: data.try_get_as_ref("content")?,
            extra: &data,
        }),
        "reply" => Ok(MsgSegmentRef::Reply {
            message_id: data.try_get_as_ref("message_id")?,
            user_id: data.try_get_as_ref("user_id")?,
            extra: &data,
        }),
        _ => Ok(MsgSegmentRef::Other { ty, extra: &data }),
    }
}

impl<'a> TryAsRef<'a, MsgSegmentRef<'a>> for MsgSegment {
    fn _try_as_ref(&'a self) -> WalleResult<MsgSegmentRef<'a>> {
        _as_ref(&self.ty, &self.data)
    }
}

impl<'a> TryAsRef<'a, MsgSegmentRef<'a>> for Value {
    fn _try_as_ref(&'a self) -> WalleResult<MsgSegmentRef<'a>> {
        if let Value::Map(m) = self {
            _as_ref(
                m.try_get_as_ref("type")?,
                m.try_get_as_ref::<&ValueMap>("data")?,
            )
        } else {
            Err(WalleError::ValueTypeNotMatch(
                "map".to_string(),
                format!("{:?}", self),
            ))
        }
    }
}

pub enum MsgSegmentMut<'a> {
    Text { text: &'a mut String },
    Mention { user_id: &'a mut String },
    Image { file_id: &'a mut String },
    Voice { file_id: &'a mut String },
    Audio { file_id: &'a mut String },
    Video { file_id: &'a mut String },
    File { file_id: &'a mut String },
    Other,
}

fn _as_mut<'a, 'b>(ty: &str, data: &'a mut ValueMap) -> WalleResult<MsgSegmentMut<'b>>
where
    'a: 'b,
{
    match ty {
        "text" => Ok(MsgSegmentMut::Text {
            text: data.try_get_as_mut("text")?,
        }),
        "mention" => Ok(MsgSegmentMut::Mention {
            user_id: data.try_get_as_mut("user_id")?,
        }),
        "image" => Ok(MsgSegmentMut::Image {
            file_id: data.try_get_as_mut("file_id")?,
        }),
        "voice" => Ok(MsgSegmentMut::Voice {
            file_id: data.try_get_as_mut("file_id")?,
        }),
        "audio" => Ok(MsgSegmentMut::Audio {
            file_id: data.try_get_as_mut("file_id")?,
        }),
        "video" => Ok(MsgSegmentMut::Video {
            file_id: data.try_get_as_mut("file_id")?,
        }),
        "file" => Ok(MsgSegmentMut::File {
            file_id: data.try_get_as_mut("file_id")?,
        }),
        _ => Ok(MsgSegmentMut::Other),
    }
}

impl<'a> TryAsMut<'a, MsgSegmentMut<'a>> for MsgSegment {
    fn _try_as_mut(&'a mut self) -> WalleResult<MsgSegmentMut<'a>> {
        _as_mut(&self.ty, &mut self.data)
    }
}

impl<'a> TryAsMut<'a, MsgSegmentMut<'a>> for Value {
    fn _try_as_mut(&'a mut self) -> WalleResult<MsgSegmentMut<'a>> {
        if let Value::Map(m) = self {
            _as_mut(
                &m.get_downcast::<String>("type")?,
                m.try_get_as_mut("data")?,
            )
        } else {
            Err(WalleError::ValueTypeNotMatch(
                "map".to_string(),
                format!("{:?}", self),
            ))
        }
    }
}
