use crate::{
    event::{
        Event as v11Event, EventContent as v11Content, MessageContent as v11MsgContent, MessageSub,
        MetaContent as v11Meta,
    },
    utils::{GroupSender, PrivateSender},
};
use std::str::FromStr;
use walle_core::{
    Event as v12Event, EventContent as v12Content, ExtendedMap, MessageEventType,
    MetaContent as v12Meta,
};

use super::WalleParseError;

impl TryFrom<v12Event> for v11Event {
    type Error = WalleParseError;

    fn try_from(event: v12Event) -> Result<Self, Self::Error> {
        let self_id = i64::from_str(&event.self_id).map_err(|e| WalleParseError::Id(e))?;
        match event.content {
            v12Content::Message(msg) => Ok(v11Event {
                time: event.time as u64,
                self_id,
                content: v11Content::Message(v11MsgContent {
                    message_id: i32::from_str(&msg.message_id)
                        .map_err(|e| WalleParseError::Id(e))?,
                    user_id: i64::from_str(&msg.user_id).map_err(|e| WalleParseError::Id(e))?,
                    message: super::message::try_parse(msg.message)?,
                    raw_message: msg.alt_message,
                    font: 0,
                    sub: match msg.ty {
                        MessageEventType::Private => MessageSub::Private {
                            sub_type: "".to_owned(),
                            sender: {
                                let mut sender = PrivateSender::default();
                                sender.user_id = i64::from_str(&msg.user_id)
                                    .map_err(|e| WalleParseError::Id(e))?;
                                sender
                            },
                        },
                        MessageEventType::Group { group_id } => MessageSub::Group {
                            group_id: i64::from_str(&group_id)
                                .map_err(|e| WalleParseError::Id(e))?,
                            sender: {
                                let mut sender = GroupSender::default();
                                sender.user_id = i64::from_str(&msg.user_id)
                                    .map_err(|e| WalleParseError::Id(e))?;
                                sender
                            },
                        },
                    },
                    extend_data: ExtendedMap::default(),
                }),
            }),

            v12Content::Meta(meta) => Ok(v11Event {
                time: event.time as u64,
                self_id,
                content: v11Content::MetaEvent(match meta {
                    v12Meta::Heartbeat {
                        status, interval, ..
                    } => v11Meta::Heartbeat {
                        status,
                        interval: interval as i64,
                    },
                }),
            }),

            v12Content::Notice(_) => {
                Err(WalleParseError::Todo("Notice Event is not implemented yet"))
            }

            v12Content::Request(_) => Err(WalleParseError::Todo(
                "Request Event is not implemented yet",
            )),
        }
    }
}

impl TryFrom<v11Event> for v12Event {
    type Error = WalleParseError;

    fn try_from(_event: v11Event) -> Result<Self, Self::Error> {
        Err(WalleParseError::Todo(
            "Parse v11Event to v12Event is not implemented yet",
        ))
    }
}
