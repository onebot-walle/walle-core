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
            v12MsgSeg::File { .. } => Err(WalleParseError::msg_seg(
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
                "v11.face" => {
                    let file = try_remove_str_from_extra_map(&mut data, "file", &ty)?;
                    Ok(v11MsgSeg::Face { file })
                }
                "v11.rps" => Ok(v11MsgSeg::Rps),
                "v11.dice" => Ok(v11MsgSeg::Dice),
                "v11.shake" => Ok(v11MsgSeg::Shake),
                "v11.anonymous" => Ok(v11MsgSeg::Anonymous),
                "v11.share" => {
                    let url = try_remove_str_from_extra_map(&mut data, "url", &ty)?;
                    let title = try_remove_str_from_extra_map(&mut data, "title", &ty)?;
                    Ok(v11MsgSeg::Share { url, title })
                }
                _ => Err(WalleParseError::msg_seg(format!(
                    "OneBot v11 don't support custom message segment type {}",
                    ty
                ))),
            },
        }
    }
}

impl TryInto<v12MsgSeg> for v11MsgSeg {
    type Error = super::WalleParseError;
    fn try_into(self) -> Result<v12MsgSeg, Self::Error> {
        match self {
            v11MsgSeg::Text { text } => Ok(v12MsgSeg::Text {
                text,
                extend: ExtendedMap::default(),
            }),
            v11MsgSeg::Face { file } => Ok(v12MsgSeg::Custom {
                ty: "v11.face".to_owned(),
                data: [("file".to_owned(), ExtendedValue::Str(file))].into(),
            }),
            v11MsgSeg::Image { file } => Ok(v12MsgSeg::Image {
                file_id: file,
                extend: ExtendedMap::default(),
            }),
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
            v11MsgSeg::Rps => Ok(v12MsgSeg::Custom {
                ty: "v11.rps".to_owned(),
                data: ExtendedMap::default(),
            }),
            v11MsgSeg::Dice => Ok(v12MsgSeg::Custom {
                ty: "v11.dice".to_owned(),
                data: ExtendedMap::default(),
            }),
            v11MsgSeg::Shake => Ok(v12MsgSeg::Custom {
                ty: "v11.shake".to_owned(),
                data: ExtendedMap::default(),
            }),
            v11MsgSeg::Poke { ty, id } => Ok(v12MsgSeg::Custom {
                ty: format!("v11.poke.{}", ty),
                data: [("id".to_owned(), ExtendedValue::Str(id))].into(),
            }),
            v11MsgSeg::Anonymous => Ok(v12MsgSeg::Custom {
                ty: "v11.anonymous".to_owned(),
                data: ExtendedMap::default(),
            }),
            v11MsgSeg::Share { url, title } => Ok(v12MsgSeg::Custom {
                ty: "v11.share".to_owned(),
                data: [
                    ("url".to_owned(), ExtendedValue::Str(url)),
                    ("title".to_owned(), ExtendedValue::Str(title)),
                ]
                .into(),
            }),
            v11MsgSeg::Contact { ty, id } => Ok(v12MsgSeg::Custom {
                ty: format!("v11.contact.{}", ty),
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
                ty: format!("v11.music.{}", ty),
                data: [("id".to_owned(), ExtendedValue::Str(id.unwrap_or_default()))].into(),
            }),
            v11MsgSeg::Reply { id } => Ok(v12MsgSeg::Reply {
                message_id: id,
                user_id: "".to_owned(),
                extend: ExtendedMap::default(),
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

pub fn try_remove_str_from_extra_map(
    map: &mut ExtendedMap,
    key: &str,
    ty: &str,
) -> Result<String, WalleParseError> {
    map.remove(key)
        .ok_or(WalleParseError::msg_seg(format!(
            "{} MessageSegment field {}",
            ty, key
        )))?
        .as_str()
        .ok_or(WalleParseError::msg_seg(format!(
            "{} MessageSegment field {} is not string",
            ty, key
        )))
}
