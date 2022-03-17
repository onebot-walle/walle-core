use serde::{Deserialize, Serialize};
use walle_core::MessageAlt;

pub type Message = Vec<MessageSegment>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum MessageSegment {
    Text {
        text: String,
    },
    Face {
        file: String,
    },
    Image {
        file: String,
    },
    Record {
        file: String,
    },
    Video {
        file: String,
    },
    At {
        qq: String,
    },
    Rps {
        value: i64,
    },
    Dice {
        value: i64,
    },
    Shake,
    Poke {
        #[serde(rename = "type")]
        ty: String,
        id: String,
    },
    Anonymous,
    Share {
        url: String,
        title: String,
    },
    Contact {
        #[serde(rename = "type")]
        ty: String,
        id: String,
    },
    Location {
        lat: String,
        lon: String,
        title: Option<String>,
        content: Option<String>,
    },
    Music {
        #[serde(rename = "type")]
        ty: String,
        id: Option<String>,
    },
    Reply {
        id: String,
    }, // todo
    Node {
        user_id: String,
        nickname: String,
        content: Vec<Self>,
    },
    Json {
        data: String,
    },
}

macro_rules! format_cq {
    ($t:expr) => {
        format!("[CQ:{}]", $t)
    };
    ($t: expr, $($k:expr, $v:expr),*) => {
        {
            let mut s = format!("[CQ:{}", $t);
            $(
                s.push_str(&format!(",{}={}", $k, escape_cq($v)));
            )*
            s.push(']');
            s
        }
    };
    ($t: expr, $($k:expr, $v:expr),*;$($ko:expr, $vo:expr),*) => {
        {
            let mut s = format!("[CQ:{}", $t);
            $(
                s.push_str(&format!(",{}={}", $k, escape_cq($v)));
            )*
            $(
                if let Some(value) = $vo {
                    s.push_str(&format!(",{}={}", $ko, escape_cq(value)))
                }
            )*
            s.push_str("]");
            s
        }
    };
}

impl MessageAlt for MessageSegment {
    fn alt(&self) -> String {
        match self {
            Self::Text { text } => escape_cq(text).replace(",", "&#44;"),
            Self::Face { file } => format_cq!("face", "file", file),
            Self::Image { file } => format_cq!("image", "file", file),
            Self::Record { file } => format_cq!("record", "file", file),
            Self::Video { file } => format_cq!("video", "file", file),
            Self::At { qq } => format_cq!("at", "qq", qq),
            Self::Rps { value } => format_cq!("rps", "value", value),
            Self::Dice { value } => format_cq!("dice", "value", value),
            Self::Shake => format_cq!("shake"),
            Self::Poke { ty, id } => format_cq!("poke", "type", ty, "id", id),
            Self::Anonymous => format_cq!("anonymous"),
            Self::Share { url, title } => format_cq!("share", "url", url, "title", title),
            Self::Contact { ty, id } => format_cq!("contact", "type", ty, "id", id),
            Self::Location {
                lat,
                lon,
                title,
                content,
            } => {
                format_cq!("location", "lat", lat, "lon", lon; "title", title, "content", content)
            }
            Self::Music { ty, id } => format_cq!("music", "type", ty; "id", id),
            Self::Reply { id } => format_cq!("reply", "id", id),
            Self::Node {
                user_id,
                nickname,
                content,
            } => format_cq!(
                "node",
                "user_id",
                user_id,
                "nickname",
                nickname,
                "content",
                &content.iter().map(|x| x.alt()).collect::<String>()
            ),
            Self::Json { data } => format_cq!("json", "data", data),
        }
    }
}

pub fn escape_cq<T: ToString>(s: &T) -> String {
    s.to_string()
        .replace("&", "&amp;")
        .replace("[", "&#91;")
        .replace("]", "&#93;")
}
