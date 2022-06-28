use super::{Message, MessageSegment};

pub trait MessageExt {
    fn extract_plain_text(&self) -> String;
}

impl MessageExt for Message {
    fn extract_plain_text(&self) -> String {
        self.iter()
            .filter_map(|seg| {
                if let MessageSegment::Text { text, .. } = seg {
                    Some(text.as_str())
                } else {
                    None
                }
            })
            .collect()
    }
}
