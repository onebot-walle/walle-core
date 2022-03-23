use super::WalleParseError;
use crate::message::MessageSegment as v11MsgSeg;
use walle_core::{ExtendedMap, ExtendedValue, MessageSegment as v12MsgSeg};

impl TryFrom<v12MsgSeg> for v11MsgSeg {
    type Error = super::WalleParseError;
    fn try_from(seg: v12MsgSeg) -> Result<Self, Self::Error> {
        match seg {
            v12MsgSeg::Text { text, .. } => Ok(v11MsgSeg::Text { text }),
            v12MsgSeg::Mention { user_id, .. } => Ok(v11MsgSeg::At { qq: user_id }),
            v12MsgSeg::MentionAll { .. } => Ok(v11MsgSeg::At {
                qq: "all".to_owned(),
            }),
            v12MsgSeg::Image { file_id, .. } => Ok(v11MsgSeg::Image { file: file_id }), //?
            v12MsgSeg::Voice { file_id, .. } => Ok(v11MsgSeg::Record { file: file_id }), //?
            v12MsgSeg::Audio { file_id, .. } => Ok(v11MsgSeg::Record { file: file_id }), //?
            v12MsgSeg::Video { file_id, .. } => Ok(v11MsgSeg::Video { file: file_id }), //?
            v12MsgSeg::File { .. } => Err(WalleParseError::Other(
                "OneBot v11 don't support file message segment".to_owned(),
            )),
            v12MsgSeg::Location {
                latitude,
                longitude,
                title,
                content,
                ..
            } => Ok(v11MsgSeg::Location {
                lat: latitude.to_string(),
                lon: longitude.to_string(),
                title: if title.is_empty() { None } else { Some(title) },
                content: if content.is_empty() {
                    None
                } else {
                    Some(content)
                },
            }),
            v12MsgSeg::Reply { message_id, .. } => Ok(v11MsgSeg::Reply { id: message_id }),
            v12MsgSeg::Custom { ty, mut data } => match ty.as_str() {
                "face" => {
                    let file = try_remove_from_extra_map(&mut data, "file", &ty)?;
                    Ok(v11MsgSeg::Face { file })
                }
                "rps" => Ok(v11MsgSeg::Rps {
                    value: try_remove_from_extra_map(&mut data, "value", &ty)?,
                }),
                "dice" => Ok(v11MsgSeg::Dice {
                    value: try_remove_from_extra_map(&mut data, "value", &ty)?,
                }),
                "json" => Ok(v11MsgSeg::Json {
                    data: try_remove_from_extra_map(&mut data, "data", &ty)?,
                }),
                "shake" => Ok(v11MsgSeg::Shake),
                "anonymous" => Ok(v11MsgSeg::Anonymous),
                "share" => {
                    let url = try_remove_from_extra_map(&mut data, "url", &ty)?;
                    let title = try_remove_from_extra_map(&mut data, "title", &ty)?;
                    Ok(v11MsgSeg::Share { url, title })
                }
                _ => Err(WalleParseError::Other(format!(
                    "OneBot v11 don't support custom message segment type {}",
                    ty
                ))),
            },
        }
    }
}

impl TryFrom<v11MsgSeg> for v12MsgSeg {
    type Error = super::WalleParseError;
    fn try_from(seg: v11MsgSeg) -> Result<Self, Self::Error> {
        match seg {
            v11MsgSeg::Text { text } => Ok(v12MsgSeg::Text {
                text,
                extend: ExtendedMap::default(),
            }),
            v11MsgSeg::Face { file } => Ok(v12MsgSeg::Custom {
                ty: "face".to_owned(),
                data: [("file".to_owned(), file.into())].into(),
            }),
            v11MsgSeg::Image { file } => Ok(image_11_to_12(file)),
            v11MsgSeg::Record { file } => Ok(v12MsgSeg::Voice {
                file_id: file,
                extend: ExtendedMap::default(),
            }),
            v11MsgSeg::Video { file } => Ok(v12MsgSeg::Video {
                file_id: file,
                extend: ExtendedMap::default(),
            }),
            v11MsgSeg::At { qq } => Ok(v12MsgSeg::Mention {
                user_id: qq,
                extend: ExtendedMap::default(),
            }),
            v11MsgSeg::Rps { value } => Ok(v12MsgSeg::Custom {
                ty: "rps".to_owned(),
                data: [("value".to_owned(), value.into())].into(),
            }),
            v11MsgSeg::Dice { value } => Ok(v12MsgSeg::Custom {
                ty: "dice".to_owned(),
                data: [("value".to_owned(), value.into())].into(),
            }),
            v11MsgSeg::Shake => Ok(v12MsgSeg::Custom {
                ty: "shake".to_owned(),
                data: ExtendedMap::default(),
            }),
            v11MsgSeg::Poke { ty, id } => Ok(v12MsgSeg::Custom {
                ty: format!("poke.{}", ty),
                data: [("id".to_owned(), ExtendedValue::Str(id))].into(),
            }),
            v11MsgSeg::Anonymous => Ok(v12MsgSeg::Custom {
                ty: "anonymous".to_owned(),
                data: ExtendedMap::default(),
            }),
            v11MsgSeg::Share { url, title } => Ok(v12MsgSeg::Custom {
                ty: "share".to_owned(),
                data: [
                    ("url".to_owned(), ExtendedValue::Str(url)),
                    ("title".to_owned(), ExtendedValue::Str(title)),
                ]
                .into(),
            }),
            v11MsgSeg::Contact { ty, id } => Ok(v12MsgSeg::Custom {
                ty: format!("contact.{}", ty),
                data: [("id".to_owned(), ExtendedValue::Str(id))].into(),
            }),
            v11MsgSeg::Location {
                lat,
                lon,
                title,
                content,
            } => Ok(v12MsgSeg::Location {
                latitude: lat.parse().unwrap(),
                longitude: lon.parse().unwrap(),
                title: title.unwrap_or_default(),
                content: content.unwrap_or_default(),
                extend: ExtendedMap::default(),
            }),
            v11MsgSeg::Music { ty, id } => Ok(v12MsgSeg::Custom {
                ty: format!("music.{}", ty),
                data: [("id".to_owned(), ExtendedValue::Str(id.unwrap_or_default()))].into(),
            }),
            v11MsgSeg::Reply { id } => Ok(v12MsgSeg::Reply {
                message_id: id,
                user_id: "".to_owned(),
                extend: ExtendedMap::default(),
            }),
            v11MsgSeg::Json { data } => Ok(v12MsgSeg::Custom {
                ty: "json".to_owned(),
                data: [("data".to_owned(), data.into())].into(),
            }),

            _ => todo!(),
        }
    }
}

pub fn try_parse<A, B>(a: Vec<A>) -> Result<Vec<B>, WalleParseError>
where
    B: TryFrom<A, Error = WalleParseError>,
{
    a.into_iter().map(|x| B::try_from(x)).collect()
}

pub fn try_remove_from_extra_map<T>(
    map: &mut ExtendedMap,
    key: &str,
    ty: &str,
) -> Result<T, WalleParseError>
where
    T: TryFrom<ExtendedValue, Error = ExtendedValue>,
{
    use walle_core::{ExtendedMapExt, WalleError};
    map.try_remove(key).map_err(|e| match e {
        WalleError::MapMissedKey(k) => WalleParseError::MsgSegMissedField(ty.to_string(), k),
        WalleError::MapValueTypeMismatch(e, g) => {
            WalleParseError::MsgSegFieldTypeMismatch(ty.to_owned(), key.to_owned(), e, g)
        }
        _ => unreachable!(),
    })
}

pub trait MessageParseExt<T> {
    type Error = WalleParseError;
    fn try_parse(self) -> Result<T, Self::Error>;
}

impl MessageParseExt<walle_core::Message> for crate::Message {
    fn try_parse(self) -> Result<walle_core::Message, WalleParseError> {
        self.into_iter().map(|s| s.try_into()).collect()
    }
}

fn image_11_to_12(file_id: String) -> v12MsgSeg {
    if let Some(b64) = file_id.strip_prefix("base64://") {
        v12MsgSeg::Image {
            file_id: [
                if b64.chars().count() > 10 {
                    &b64.split_at(10).0
                } else {
                    ""
                },
                "...",
            ]
            .concat(),
            extend: [("url".to_string(), file_id.into())].into(),
        }
    } else if let Some(path) = file_id.strip_prefix("file:///") {
        v12MsgSeg::Image {
            file_id: path.to_string(),
            extend: [("url".to_string(), file_id.into())].into(),
        }
    } else if file_id.starts_with("http://") || file_id.starts_with("https://") {
        v12MsgSeg::Image {
            file_id: "".to_string(),
            extend: [("url".to_string(), file_id.into())].into(),
        }
    } else {
        v12MsgSeg::Image {
            file_id,
            extend: ExtendedMap::default(),
        }
    }
}
