use std::collections::HashMap;

use serde::{de::Visitor, ser::SerializeMap, Deserialize, Serialize};

use crate::EmptyContent;

/// 在事件和动作参数中用于表示聊天消息的数据类型
pub type Message = Vec<MessageSegment>;

/// 消息段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum OldMessageSegment {
    Text {
        text: String,
    },
    Mention {
        user_id: String,
    },
    MentionAll,
    Image {
        file_id: String,
    },
    Voice {
        file_id: String,
    },
    Audio {
        file_id: String,
    },
    Video {
        file_id: String,
    },
    File {
        file_id: String,
    },
    Location {
        latitude: f64,
        longitude: f64,
        title: String,
        content: String,
    },
    Reply {
        message_id: String,
        user_id: String,
    },
}

/// Message 构建 trait
pub trait MessageBuild {
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
    fn custom(self, ty: String, data: HashMap<String, MessageValue>) -> Self;
}

impl MessageBuild for Message {
    fn text(mut self, text: String) -> Self {
        self.push(MessageSegment::Text { text });
        self
    }
    fn mention(mut self, user_id: String) -> Self {
        self.push(MessageSegment::Mention { user_id });
        self
    }
    fn mention_all(mut self) -> Self {
        self.push(MessageSegment::MentionAll);
        self
    }
    fn image(mut self, file_id: String) -> Self {
        self.push(MessageSegment::Image { file_id });
        self
    }
    fn voice(mut self, file_id: String) -> Self {
        self.push(MessageSegment::Voice { file_id });
        self
    }
    fn audio(mut self, file_id: String) -> Self {
        self.push(MessageSegment::Audio { file_id });
        self
    }
    fn video(mut self, file_id: String) -> Self {
        self.push(MessageSegment::Video { file_id });
        self
    }
    fn file(mut self, file_id: String) -> Self {
        self.push(MessageSegment::File { file_id });
        self
    }
    fn location(mut self, latitude: f64, longitude: f64, title: String, content: String) -> Self {
        self.push(MessageSegment::Location {
            latitude,
            longitude,
            title,
            content,
        });
        self
    }
    fn reply(mut self, message_id: String, user_id: String) -> Self {
        self.push(MessageSegment::Reply {
            message_id,
            user_id,
        });
        self
    }
    fn custom(mut self, ty: String, data: HashMap<String, MessageValue>) -> Self {
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
            Self::Text { text } => text.to_owned(),
            Self::Mention { user_id } => format!("[Mention={}]", user_id),
            Self::MentionAll => "[MentionAll]".to_owned(),
            Self::Image { file_id: _ } => "[Image]".to_owned(),
            Self::Voice { file_id: _ } => "[Voice]".to_owned(),
            Self::Audio { file_id: _ } => "[Audio]".to_owned(),
            Self::Video { file_id: _ } => "[Video]".to_owned(),
            Self::File { file_id: _ } => "[File]".to_owned(),
            Self::Location {
                latitude: _,
                longitude: _,
                title: _,
                content: _,
            } => "[Location]".to_owned(),
            Self::Reply {
                message_id: _,
                user_id,
            } => format!("[Reply={}]", user_id),
            Self::Custom { ty, data: _ } => format!("[{}]", ty),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageSegment {
    Text {
        text: String,
    },
    Mention {
        user_id: String,
    },
    MentionAll,
    Image {
        file_id: String,
    },
    Voice {
        file_id: String,
    },
    Audio {
        file_id: String,
    },
    Video {
        file_id: String,
    },
    File {
        file_id: String,
    },
    Location {
        latitude: f64,
        longitude: f64,
        title: String,
        content: String,
    },
    Reply {
        message_id: String,
        user_id: String,
    },
    Custom {
        ty: String,
        data: HashMap<String, MessageValue>,
    },
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
#[serde(untagged)]
pub enum MessageValue {
    Str(String),
    F64(f64),
    Int(i64),
    Bool(bool),
}

impl MessageValue {
    pub fn as_str(self) -> Option<String> {
        match self {
            Self::Str(v) => Some(v),
            _ => None,
        }
    }
    pub fn as_f64(self) -> Option<f64> {
        match self {
            Self::F64(v) => Some(v),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn as_int(self) -> Option<i64> {
        match self {
            Self::Int(v) => Some(v),
            _ => None,
        }
    }
    #[allow(dead_code)]
    pub fn as_bool(self) -> Option<bool> {
        match self {
            Self::Bool(v) => Some(v),
            _ => None,
        }
    }
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
        }

        match self {
            Self::Text { text } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "text")?;
                map.serialize_entry("data", &{
                    let mut datamap = HashMap::new();
                    datamap.insert("text", text);
                    datamap
                })?;
                map.end()
            }
            Self::Mention { user_id } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "mention")?;
                map.serialize_entry("data", &{
                    let mut datamap = HashMap::new();
                    datamap.insert("user_id", user_id);
                    datamap
                })?;
                map.end()
            }
            Self::MentionAll => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "mention_all")?;
                map.serialize_entry("data", &EmptyContent::default())?;
                map.end()
            }
            Self::Image { file_id } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "image")?;
                map.serialize_entry("data", &{
                    let mut datamap = HashMap::new();
                    datamap.insert("file_id", file_id);
                    datamap
                })?;
                map.end()
            }
            Self::Voice { file_id } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "voice")?;
                map.serialize_entry("data", &{
                    let mut datamap = HashMap::new();
                    datamap.insert("file_id", file_id);
                    datamap
                })?;
                map.end()
            }
            Self::Audio { file_id } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "audio")?;
                map.serialize_entry("data", &{
                    let mut datamap = HashMap::new();
                    datamap.insert("file_id", file_id);
                    datamap
                })?;
                map.end()
            }
            Self::Video { file_id } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "video")?;
                map.serialize_entry("data", &{
                    let mut datamap = HashMap::new();
                    datamap.insert("file_id", file_id);
                    datamap
                })?;
                map.end()
            }
            Self::File { file_id } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "file")?;
                map.serialize_entry("data", &{
                    let mut datamap = HashMap::new();
                    datamap.insert("file_id", file_id);
                    datamap
                })?;
                map.end()
            }
            Self::Location {
                latitude,
                longitude,
                title,
                content,
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
                    },
                )?;
                map.end()
            }
            Self::Reply {
                message_id,
                user_id,
            } => {
                let mut map = serializer.serialize_map(Some(2))?;
                map.serialize_entry("type", "location")?;
                map.serialize_entry("data", &{
                    let mut datamap = HashMap::new();
                    datamap.insert("message_id", message_id);
                    datamap.insert("user_id", user_id);
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
        let mut ty = None;
        let mut data: Option<HashMap<String, MessageValue>> = None;
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
                if let Some(text) = data.remove("text") {
                    Ok(Self::Value::Text {
                        text: text.as_str().unwrap(),
                    })
                } else {
                    Err(serde::de::Error::missing_field("text"))
                }
            }
            "mention" => {
                let user_id = data
                    .remove("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("user_id"))?;
                Ok(Self::Value::Mention { user_id })
            }
            "mention_all" => Ok(Self::Value::MentionAll),
            "image" => {
                let file_id = data
                    .remove("file_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("file_id"))?;
                Ok(Self::Value::Image { file_id })
            }
            "voice" => {
                let file_id = data
                    .remove("file_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("file_id"))?;
                Ok(Self::Value::Voice { file_id })
            }
            "audio" => {
                let file_id = data
                    .remove("file_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("file_id"))?;
                Ok(Self::Value::Audio { file_id })
            }
            "video" => {
                let file_id = data
                    .remove("file_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("file_id"))?;
                Ok(Self::Value::Video { file_id })
            }
            "file" => {
                let file_id = data
                    .remove("file_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("file_id"))?;
                Ok(Self::Value::File { file_id })
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
                let title = data
                    .remove("title")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("title"))?;
                let content = data
                    .remove("content")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("content"))?;
                Ok(Self::Value::Location {
                    latitude,
                    longitude,
                    title,
                    content,
                })
            }
            "reply" => {
                let message_id = data
                    .remove("message_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("message_id"))?;
                let user_id = data
                    .remove("user_id")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| serde::de::Error::missing_field("user_id"))?;
                Ok(Self::Value::Reply {
                    message_id,
                    user_id,
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
