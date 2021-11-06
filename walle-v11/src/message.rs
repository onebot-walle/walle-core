use serde::{Deserialize, Serialize};

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
    Rps,
    Dice,
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
    // todo
}

pub trait MessageTo12 {
    fn to_12(self) -> walle_core::Message;
}

impl MessageTo12 for Message {
    fn to_12(self) -> walle_core::Message {
        // walle_core::Message::new()
        todo!()
    }
}

pub trait MessageTo11 {
    fn to_11(self) -> Message;
}

impl MessageTo11 for walle_core::Message {
    fn to_11(self) -> Message {
        todo!()
    }
}
