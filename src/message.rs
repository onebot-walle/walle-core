use serde::{Deserialize, Serialize};

/// 在事件和动作参数中用于表示聊天消息的数据类型
pub type Message = Vec<MessageSegment>;

/// 消息段
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum MessageSegment {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "mention")]
    Mention { user_id: String },
    #[serde(rename = "mention_all")]
    MentionAll,
    #[serde(rename = "image")]
    Image { file_id: String },
    #[serde(rename = "voice")]
    Voice { file_id: String },
    #[serde(rename = "audio")]
    Audio { file_id: String },
    #[serde(rename = "video")]
    Video { file_id: String },
    #[serde(rename = "file")]
    File { file_id: String },
    #[serde(rename = "location")]
    Location {
        latitude: f64,
        longitude: f64,
        title: String,
        content: String,
    },
    #[serde(rename = "reply")]
    Reply { message_id: String, user_id: String },
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
}
