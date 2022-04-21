use crate::MessageSegment;

use super::Message;

pub trait IntoMessage {
    fn into_message(self) -> Message;
}

impl IntoMessage for String {
    fn into_message(self) -> Message {
        vec![MessageSegment::text(self)]
    }
}

impl IntoMessage for &str {
    fn into_message(self) -> Message {
        vec![MessageSegment::text(self.to_string())]
    }
}

impl IntoMessage for Message {
    fn into_message(self) -> Message {
        self
    }
}
