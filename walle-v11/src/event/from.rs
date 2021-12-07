use walle_core::{
    event::{
        Message as v12MessageEventContent, MessageEventType as v12MessageEventType,
        Meta as v12MetaContent,
    },
    Event as v12Evnet, EventContent as v12EventContent,
};

use super::{EventContent, MessageSub};
use crate::message::MessageTo12;

static IMPL: &str = "Walle-v11";
static PLATFORM: &str = "v11";

// impl Into<v12Evnet> for super::Event {
//     fn into(self) -> v12Evnet {
//         match self.content {
//             EventContent::Message(m) => {
//                 v12Evnet {
//                     id: format!("{}{}", self.self_id, self.time), //todo
//                     time: self.time as u64,
//                     r#impl: IMPL.to_owned(),
//                     platform: PLATFORM.to_owned(),
//                     content: v12EventContent::Message(v12MessageEventContent {
//                         ty: match m.sub {
//                             MessageSub::Group {
//                                 group_id,
//                                 sender: _,
//                             } => v12MessageEventType::Group {
//                                 group_id: format!("{}", group_id),
//                             },
//                             MessageSub::Private {
//                                 sender: _,
//                                 sub_type: _,
//                             } => v12MessageEventType::Private,
//                         },
//                         message_id: format!("{}{}", self.self_id, self.time), //todo
//                         message: m.message.to_12(),
//                         alt_message: m.raw_message,
//                         user_id: format!("{}", m.user_id),
//                         sub_type: "".to_owned(),
//                     }),
//                     self_id: self.self_id.to_string(),
//                 }
//             }
//             EventContent::Notice(_n) => {
//                 todo!()
//             }
//             EventContent::Request(_r) => {
//                 todo!()
//             }
//             EventContent::MetaEvent(m) => match m {
//                 super::MetaEvent::Lifecycle { sub_type: _ } => todo!(),
//                 super::MetaEvent::Heartbeat { status, interval } => {
//                     v12Evnet {
//                         id: format!("{}{}", self.self_id, self.time), //todo
//                         time: self.time as u64,
//                         r#impl: IMPL.to_owned(),
//                         platform: PLATFORM.to_owned(),
//                         self_id: self.self_id.to_string(),
//                         content: v12EventContent::Meta(v12MetaContent::Heartbeat {
//                             status: status,
//                             interval: interval as u32,
//                             sub_type: "".to_owned(),
//                         }),
//                     }
//                 }
//             },
//         }
//     }
// }
