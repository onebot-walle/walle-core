use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{
    event::{
        self, new_event, Connect, DetailTypeLevel, Event, Group, Heartbeat, Message, MessageEvent,
        Meta, Private,
    },
    prelude::{MsgSegment, Selft, Version},
    segment::{Image, Mention, Reply, Text},
    util::{new_uuid, PushToValueMap},
    v11::{self, V11Event, V11MsgSegment, V11MsgType},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]

pub enum ComEvent {
    V12Event(Event),
    V11Event(V11Event),
}

impl ComEvent {
    pub fn to_v11(self) -> V11Event {
        dbg!(self.clone());
        match self {
            ComEvent::V12Event(_) => todo!(),
            ComEvent::V11Event(event) => event,
        }
    }
    pub fn to_v12(self) -> Event {
        match self {
            ComEvent::V12Event(event) => event,
            ComEvent::V11Event(event) => {
                dbg!(event.clone());
                let event: Event = match event.post_type {
                    v11::Post::Meta(meta_event) => match meta_event {
                        v11::MetaEvent::HeartBeatEvent {
                            interval,
                            sub_type,
                            status,
                        } => new_event(
                            new_uuid(),
                            event.time as f64,
                            Meta,
                            Heartbeat { interval },
                            (),
                            (),
                            (),
                            HashMap::with_capacity(0),
                        )
                        .into(),
                        v11::MetaEvent::LifecycleEvent { sub_type, status } => new_event(
                            new_uuid(),
                            event.time as f64,
                            Meta,
                            // TODO assert sub_type = connect
                            Connect {
                                version: Version {
                                    implt: "walle-adapter".into(),
                                    version: crate::VERSION.into(),
                                    onebot_version: "12".into(),
                                },
                            },
                            (),
                            (),
                            (),
                            HashMap::with_capacity(0),
                        )
                        .into(),
                    },
                    v11::Post::Message(message) => match message {
                        v11::MessageEvent::PrivateMessage {
                            sub_type,
                            message_id,
                            user_id,
                            message,
                            raw_message,
                            sender,
                            target_id,
                            temp_source,
                            peer_id,
                        } => new_event(
                            new_uuid(),
                            event.time as f64,
                            Message {
                                selft: Selft {
                                    platform: "walle-adapter".into(), // TODO cache platform
                                    user_id: event.self_id.to_string(),
                                },
                                message_id: message_id.to_string(),
                                message: message
                                    .into_iter()
                                    .map(|msg| msg.trans(&sender.user_id.to_string()))
                                    .collect(),
                                alt_message: raw_message,
                                user_id: user_id.to_string(),
                            },
                            Private,
                            (),
                            (),
                            (),
                            HashMap::with_capacity(0),
                        )
                        .into(),
                        v11::MessageEvent::GroupMessage {
                            sub_type,
                            message_id,
                            user_id,
                            message,
                            raw_message,
                            sender,
                            group_id,
                            target_id,
                            temp_source,
                            peer_id,
                        } => new_event(
                            new_uuid(),
                            event.time as f64,
                            Message {
                                selft: Selft {
                                    platform: "walle-adapter".into(), // TODO cache platform
                                    user_id: event.self_id.to_string(),
                                },
                                message_id: message_id.to_string(),
                                message: message
                                    .into_iter()
                                    .map(|msg| msg.trans(&sender.user_id.to_string()))
                                    .collect(),
                                alt_message: raw_message,
                                user_id: user_id.to_string(),
                            },
                            Group {
                                group_id: group_id.unwrap_or_default().to_string(),
                            },
                            (),
                            (),
                            (),
                            HashMap::with_capacity(0),
                        )
                        .into(),
                    },

                    v11::Post::MessageSent(_) => {
                        todo!()
                    }
                    v11::Post::Notice(_) => {
                        todo!()
                    }
                    v11::Post::Request(_) => {
                        todo!()
                    }
                };
                dbg!(event.clone());
                return event;
            }
        }
    }
}

impl From<event::Event> for ComEvent {
    fn from(value: event::Event) -> Self {
        Self::V12Event(value)
    }
}
impl From<v11::V11Event> for ComEvent {
    fn from(value: v11::V11Event) -> Self {
        Self::V11Event(value)
    }
}

impl V11MsgSegment {
    pub fn trans(self, sender_id: &String) -> MsgSegment {
        match self {
            V11MsgSegment::Text { text } => Text { text }.into(),
            V11MsgSegment::Face { id } => todo!(),
            V11MsgSegment::Image { file, ty, url } => Image { file_id: file }.into(),
            V11MsgSegment::At { qq } => Mention { user_id: qq }.into(),
            V11MsgSegment::Poke { ty, id, name } => todo!(),
            V11MsgSegment::Reply { id } => Reply {
                message_id: id.to_string(),
                user_id: None,
            }
            .into(),
        }
    }
}
