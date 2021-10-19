use serde::Serialize;
mod action;
mod action_resp;
mod event;
mod event_builder;
mod message;

pub use action::Action;
pub use action_resp::*;
pub use event::{Events, MessageEvent, MetaEvent, NoticeEvent, RequestEvent};
pub use message::{Message, MessageSegment};

pub trait ToJson: Serialize {
    fn json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }
}

