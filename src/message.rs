use std::collections::HashMap;

use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

use crate::utils::{ExtendedMap, ExtendedValue};

/// 在事件和动作参数中用于表示聊天消息的数据类型
pub type Message = Vec<MessageSegment>;

// 消息段
// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
// #[serde(tag = "type", content = "data")]
// #[serde(rename_all = "snake_case")]
// pub enum OldMessageSegment {
//     Text {
//         text: String,
//     },
//     Mention {
//         user_id: String,
//     },
//     MentionAll,
//     Image {
//         file_id: String,
//     },
//     Voice {
//         file_id: String,
//     },
//     Audio {
//         file_id: String,
//     },
//     Video {
//         file_id: String,
//     },
//     File {
//         file_id: String,
//     },
//     Location {
//         latitude: f64,
//         longitude: f64,
//         title: String,
//         content: String,
//     },
//     Reply {
//         message_id: String,
//         user_id: String,
//     },
// }

/// Message 构建 trait
pub trait MessageBuild {
    fn text_with_extend(self, text: String, extend: ExtendedMap) -> Self;
    fn mention_with_extend(self, user_id: String, extend: ExtendedMap) -> Self;
    fn mention_all_with_extend(self, extend: ExtendedMap) -> Self;
    fn image_with_extend(self, file_id: String, extend: ExtendedMap) -> Self;
    fn voice_with_extend(self, file_id: String, extend: ExtendedMap) -> Self;
    fn audio_with_extend(self, file_id: String, extend: ExtendedMap) -> Self;
    fn video_with_extend(self, file_id: String, extend: ExtendedMap) -> Self;
    fn file_with_extend(self, file_id: String, extend: ExtendedMap) -> Self;
    fn location_with_extend(
        self,
        latitude: f64,
        longitude: f64,
        title: String,
        content: String,
        extend: ExtendedMap,
    ) -> Self;
    fn reply_with_extend(self, message_id: String, user_id: String, extend: ExtendedMap) -> Self;

    fn text(self, text: String) -> Self;
    fn mention(self, user_id: String) -> Self;
    fn mention_all(self) -> Self;
    fn image(self, file_id: String) -> Self;
    fn voice(self, file_id: String) -> Self;
    fn audio(self, file_id: String) -> Self;
    fn video(self, file_id: String) -> Self;
    fn file(self, file_id: String) -> Self;
    fn location(self, latitude: f64, longitude: f64, title: String, content: String) -> Self;
    fn reply(self, message_id: String, user_id: String) -> Self;
    fn custom(self, ty: String, data: ExtendedMap) -> Self;
}

impl MessageBuild for Message {
    fn text_with_extend(mut self, text: String, extend: ExtendedMap) -> Self {
        self.push(MessageSegment::Text { text, extend });
        self
    }
    fn mention_with_extend(mut self, user_id: String, extend: ExtendedMap) -> Self {
        self.push(MessageSegment::Mention { user_id, extend });
        self
    }
    fn mention_all_with_extend(mut self, extend: ExtendedMap) -> Self {
        self.push(MessageSegment::MentionAll { extend });
        self
    }
    fn file_with_extend(mut self, file_id: String, extend: ExtendedMap) -> Self {
        self.push(MessageSegment::File { file_id, extend });
        self
    }
    fn image_with_extend(mut self, file_id: String, extend: ExtendedMap) -> Self {
        self.push(MessageSegment::Image { file_id, extend });
        self
    }
    fn voice_with_extend(mut self, file_id: String, extend: ExtendedMap) -> Self {
        self.push(MessageSegment::Voice { file_id, extend });
        self
    }
    fn audio_with_extend(mut self, file_id: String, extend: ExtendedMap) -> Self {
        self.push(MessageSegment::Audio { file_id, extend });
        self
    }
    fn video_with_extend(mut self, file_id: String, extend: ExtendedMap) -> Self {
        self.push(MessageSegment::Video { file_id, extend });
        self
    }
    fn location_with_extend(
        mut self,
        latitude: f64,
        longitude: f64,
        title: String,
        content: String,
        extend: ExtendedMap,
    ) -> Self {
        self.push(MessageSegment::Location {
            latitude,
            longitude,
            title,
            content,
            extend,
        });
        self
    }
    fn reply_with_extend(
        mut self,
        message_id: String,
        user_id: String,
        extend: ExtendedMap,
    ) -> Self {
        self.push(MessageSegment::Reply {
            message_id,
            user_id,
            extend,
        });
        self
    }
    fn text(self, text: String) -> Self {
        self.text_with_extend(text, ExtendedMap::new())
    }
    fn mention(self, user_id: String) -> Self {
        self.mention_with_extend(user_id, ExtendedMap::new())
    }
    fn mention_all(self) -> Self {
        self.mention_all_with_extend(ExtendedMap::new())
    }
    fn image(self, file_id: String) -> Self {
        self.image_with_extend(file_id, ExtendedMap::new())
    }
    fn voice(self, file_id: String) -> Self {
        self.voice_with_extend(file_id, ExtendedMap::new())
    }
    fn audio(self, file_id: String) -> Self {
        self.audio_with_extend(file_id, ExtendedMap::new())
    }
    fn video(self, file_id: String) -> Self {
        self.video_with_extend(file_id, ExtendedMap::new())
    }
    fn file(self, file_id: String) -> Self {
        self.file_with_extend(file_id, ExtendedMap::new())
    }
    fn location(self, latitude: f64, longitude: f64, title: String, content: String) -> Self {
        self.location_with_extend(latitude, longitude, title, content, ExtendedMap::new())
    }
    fn reply(self, message_id: String, user_id: String) -> Self {
        self.reply_with_extend(message_id, user_id, ExtendedMap::new())
    }
    fn custom(mut self, ty: String, data: ExtendedMap) -> Self {
        self.push(MessageSegment::Custom { ty, data });
        self
    }
}

pub trait MessageAlt {
    fn alt(&self) -> String;
}

impl MessageAlt for Message {
    fn alt(&self) -> String {
        let mut alt = String::new();
        for seg in self {
            alt.push_str(&seg.alt())
        }
        alt
    }
}

impl MessageAlt for MessageSegment {
    fn alt(&self) -> String {
        match self {
            Self::Text { text, extend: _ } => text.to_owned(),
            Self::Mention { user_id, extend: _ } => format!("[Mention={}]", user_id),
            Self::MentionAll { extend: _ } => "[MentionAll]".to_owned(),
            Self::Image {
                file_id: _,
                extend: _,
            } => "[Image]".to_owned(),
            Self::Voice {
                file_id: _,
                extend: _,
            } => "[Voice]".to_owned(),
            Self::Audio {
                file_id: _,
                extend: _,
            } => "[Audio]".to_owned(),
            Self::Video {
                file_id: _,
                extend: _,
            } => "[Video]".to_owned(),
            Self::File {
                file_id: _,
                extend: _,
            } => "[File]".to_owned(),
            Self::Location {
                latitude: _,
                longitude: _,
                title: _,
                content: _,
                extend: _,
            } => "[Location]".to_owned(),
            Self::Reply {
                message_id: _,
                user_id,
                extend: _,
            } => format!("[Reply={}]", user_id),
            Self::Custom { ty, data: _ } => format!("[{}]", ty),
        }
    }
}

/// 消息段
#[derive(Debug, Clone, PartialEq)]
pub enum MessageSegment {
    Text {
        text: String,
        extend: ExtendedMap,
    },
    Mention {
        user_id: String,
        extend: ExtendedMap,
    },
    MentionAll {
        extend: ExtendedMap,
    },
    Image {
        file_id: String,
        extend: ExtendedMap,
    },
    Voice {
        file_id: String,
        extend: ExtendedMap,
    },
    Audio {
        file_id: String,
        extend: ExtendedMap,
    },
    Video {
        file_id: String,
        extend: ExtendedMap,
    },
    File {
        file_id: String,
        extend: ExtendedMap,
    },
    Location {
        latitude: f64,
        longitude: f64,
        title: String,
        content: String,
        extend: ExtendedMap,
    },
    Reply {
        message_id: String,
        user_id: String,
        extend: ExtendedMap,
    },
    Custom {
        ty: String,
        data: ExtendedMap,
    },
}

impl Serialize for MessageSegment {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct Location<'a> {
            latitude: &'a f64,
            longitude: &'a f64,
            title: &'a str,
            content: &'a str,
            #[serde(flatten)]
            extend: &'a ExtendedMap,
        }

        fn smap<S>(
            serializer: S,
            ty: &str,
            key_word: &str,
            value: &str,
            extended_map: &ExtendedMap,
        ) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            let mut map = serializer.serialize_map(Some(2))?;
            map.serialize_entry("type", ty)?;
            let v = ExtendedValue::Str(value.to_owned());
            map.serialize_entry("data", &{
                let mut datamap = HashMap::new();
                datamap.insert(key_word, &v);
                for (key, value) in extended_map {
                    datamap.insert(key, value);
                }
                datamap
            })?;
            map.end()
        }

        match self {
            Self::Text { text, extend } => smap(serializer, "text", "text", text, extend),
            Self::Mention { user_id, extend } => {
                smap(serializer, "mention", "user_id", user_id, extend)
            }
            Self::MentionAll { extend } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "mention_all")?;
                map.serialize_entry("data", &extend)?;
                map.end()
            }
            Self::Image { file_id, extend } => {
                smap(serializer, "image", "file_id", file_id, extend)
            }
            Self::Voice { file_id, extend } => {
                smap(serializer, "voice", "file_id", file_id, extend)
            }
            Self::Audio { file_id, extend } => {
                smap(serializer, "audio", "file_id", file_id, extend)
            }
            Self::Video { file_id, extend } => {
                smap(serializer, "video", "file_id", file_id, extend)
            }
            Self::File { file_id, extend } => smap(serializer, "file", "file_id", file_id, extend),
            Self::Location {
                latitude,
                longitude,
                title,
                content,
                extend,
            } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "location")?;
                map.serialize_entry(
                    "data",
                    &Location {
                        latitude,
                        longitude,
                        title,
                        content,
                        extend,
                    },
                )?;
                map.end()
            }
            Self::Reply {
                message_id,
                user_id,
                extend,
            } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "location")?;
                map.serialize_entry("data", &{
                    let mut datamap = HashMap::new();
                    datamap.insert("message_id", ExtendedValue::Str(message_id.to_owned()));
                    datamap.insert("user_id", ExtendedValue::Str(user_id.to_owned()));
                    for (key, value) in extend {
                        datamap.insert(key, value.clone());
                    }
                    datamap
                })?;
                map.end()
            }
            Self::Custom { ty, data } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", ty)?;
                map.serialize_entry("data", data)?;
                map.end()
            }
        }
    }
}

struct MSVister;

#[derive(Deserialize)]
#[serde(field_identifier, rename_all = "lowercase")]
enum Field {
    Type,
    Data,
}

impl<'de> Visitor<'de> for MSVister {
    type Value = MessageSegment;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("should be a message")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        fn get_data<'de, A>(
            map: &mut ExtendedMap,
            key_word: &'static str,
        ) -> Result<String, A::Error>
        where
            A: serde::de::MapAccess<'de>,
        {
            map.remove(key_word)
                .and_then(|v| v.as_str())
                .ok_or_else(|| serde::de::Error::missing_field(key_word))
        }

        let mut ty = None;
        let mut data: Option<ExtendedMap> = None;
        while let Some(key) = map.next_key()? {
            match key {
                Field::Type => ty = Some(map.next_value()?),
                Field::Data => data = Some(map.next_value()?),
            }
        }
        let ty = ty.ok_or_else(|| serde::de::Error::missing_field("type"))?;
        let mut data = data.ok_or_else(|| serde::de::Error::missing_field("data"))?;
        match ty {
            "text" => {
                let text = get_data::<A>(&mut data, "text")?;
                Ok(MessageSegment::Text { text, extend: data })
            }
            "mention" => {
                let user_id = get_data::<A>(&mut data, "user_id")?;
                Ok(Self::Value::Mention {
                    user_id,
                    extend: data,
                })
            }
            "mention_all" => Ok(Self::Value::MentionAll { extend: data }),
            "image" => {
                let file_id = get_data::<A>(&mut data, "file_id")?;
                Ok(Self::Value::Image {
                    file_id,
                    extend: data,
                })
            }
            "voice" => {
                let file_id = get_data::<A>(&mut data, "file_id")?;
                Ok(Self::Value::Voice {
                    file_id,
                    extend: data,
                })
            }
            "audio" => {
                let file_id = get_data::<A>(&mut data, "file_id")?;
                Ok(Self::Value::Audio {
                    file_id,
                    extend: data,
                })
            }
            "video" => {
                let file_id = get_data::<A>(&mut data, "file_id")?;
                Ok(Self::Value::Video {
                    file_id,
                    extend: data,
                })
            }
            "file" => {
                let file_id = get_data::<A>(&mut data, "file_id")?;
                Ok(Self::Value::File {
                    file_id,
                    extend: data,
                })
            }
            "location" => {
                let latitude = data
                    .remove("latitude")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| serde::de::Error::missing_field("latitude"))?;
                let longitude = data
                    .remove("longitude")
                    .and_then(|v| v.as_f64())
                    .ok_or_else(|| serde::de::Error::missing_field("longitude"))?;
                let title = get_data::<A>(&mut data, "title")?;
                let content = get_data::<A>(&mut data, "content")?;
                Ok(Self::Value::Location {
                    latitude,
                    longitude,
                    title,
                    content,
                    extend: data,
                })
            }
            "reply" => {
                let message_id = get_data::<A>(&mut data, "message_id")?;
                let user_id = get_data::<A>(&mut data, "user_id")?;
                Ok(Self::Value::Reply {
                    message_id,
                    user_id,
                    extend: data,
                })
            }
            _ => Ok(Self::Value::Custom {
                ty: ty.to_owned(),
                data,
            }),
        }
    }
}

impl<'de> Deserialize<'de> for MessageSegment {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_map(MSVister)
    }
}
