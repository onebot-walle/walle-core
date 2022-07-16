#[test]
fn event() {
    use crate::alt::ColoredAlt;
    use crate::event::*;
    use crate::value_map;
    use crate::prelude::WalleError;

    fn test<T>(event: (&str, Event, T))
    where
        T: TryFrom<Event, Error = WalleError> + std::fmt::Debug + PartialEq,
    {
        assert_eq!(serde_json::from_str::<Event>(event.0).unwrap(), event.1);
        assert_eq!(
            serde_json::from_str::<Event>(&serde_json::to_string(&event.1).unwrap()).unwrap(),
            event.1
        );
        assert_eq!(T::try_from(event.1.clone()).unwrap(), event.2);
        println!("{}", event.1.colored_alt());
    }

    test((
        r#"{
                "id": "b6e65187-5ac0-489c-b431-53078e9d2bbb",
                "impl": "go_onebot_qq",
                "platform": "qq",
                "self_id": "123234",
                "time": 1632847927,
                "type": "message",
                "detail_type": "private",
                "sub_type": "",
                "message_id": "6283",
                "message": [
                    {
                        "type": "text",
                        "data": {
                            "text": "OneBot is not a bot"
                        }
                    },
                    {
                        "type": "image",
                        "data": {
                            "file_id": "e30f9684-3d54-4f65-b2da-db291a477f16"
                        }
                    }
                ],
                "alt_message": "OneBot is not a bot[图片]",
                "user_id": "123456788"
            }"#,
        Event {
            id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_string(),
            implt: "go_onebot_qq".to_string(),
            platform: "qq".to_string(),
            self_id: "123234".to_string(),
            time: 1632847927.0,
            ty: "message".to_string(),
            detail_type: "private".to_string(),
            sub_type: "".to_string(),
            extra: value_map! {
                "message_id": "6283",
                "message": [
                    {
                        "type": "text",
                        "data": {
                            "text": "OneBot is not a bot"
                        }
                    },
                    {
                        "type": "image",
                        "data": {
                            "file_id": "e30f9684-3d54-4f65-b2da-db291a477f16"
                        }
                    }
                ],
                "alt_message": "OneBot is not a bot[图片]",
                "user_id": "123456788"
            },
        },
        new_event(
            "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_string(),
            1632847927.0,
            "123234".to_string(),
            Message {
                message: vec![
                    crate::segment::MessageSegment {
                        ty: "text".to_string(),
                        data: value_map! {
                            "text": "OneBot is not a bot"
                        },
                    },
                    crate::segment::MessageSegment {
                        ty: "image".to_string(),
                        data: value_map! {
                            "file_id": "e30f9684-3d54-4f65-b2da-db291a477f16"
                        },
                    },
                ],
                message_id: "6283".to_string(),
                alt_message: "OneBot is not a bot[图片]".to_string(),
                user_id: "123456788".to_string(),
            },
            Private {},
            (),
            (),
            (),
            value_map!(),
        ),
    ));
}

// #[test]
// fn action() {
//     use crate::action::{GetLatestEvents, SendMessage};
//     use crate::prelude::*;
//     use crate::util::Echo;
//     use std::collections::HashMap;

//     let data = vec![
//         (
//             r#"{
//             "action": "get_latest_events",
//             "params": {
//                 "limit": 100,
//                 "timeout": 0
//             }
//         }"#,
//             StandardAction::GetLatestEvents(GetLatestEvents {
//                 limit: 100,
//                 timeout: 0,
//                 extra: [].into(),
//             }),
//         ),
//         (
//             r#"{
//                 "action": "send_message",
//                 "params": {
//                     "detail_type": "group",
//                     "group_id": "12467",
//                     "message": "我是文字巴拉巴拉巴拉"
//                 }
//             }"#,
//             StandardAction::SendMessage(SendMessage {
//                 detail_type: "group".to_owned(),
//                 group_id: Some("12467".to_owned()),
//                 user_id: None,
//                 guild_id: None,
//                 channel_id: None,
//                 message: vec![MessageSegment::Text {
//                     text: "我是文字巴拉巴拉巴拉".to_owned(),
//                     extra: HashMap::new(),
//                 }],
//                 extra: [].into(),
//             }),
//         ),
//         (
//             r#"{
//                 "action": "get_self_info",
//                 "params": {}
//             }"#,
//             StandardAction::GetSelfInfo([].into()),
//         ),
//     ];

//     use crate::alt::ColoredAlt;
//     for (json, action) in data {
//         assert_eq!(
//             serde_json::from_str::<Echo<StandardAction>>(json)
//                 .unwrap()
//                 .unpack()
//                 .0,
//             action
//         );
//         if let Some(alt) = action.colored_alt() {
//             println!("{}", alt);
//         }
//         let json_str = serde_json::to_string(&action).unwrap();
//         assert_eq!(
//             serde_json::from_str::<StandardAction>(&json_str).unwrap(),
//             action
//         );
//     }
// }

// #[test]
// fn action_resp() {
//     use crate::prelude::*;
//     use crate::resp::StatusContent;
//     use crate::util::{ExtendedMap, ExtendedValue};
//     use std::collections::HashMap;

//     let status_data = (
//         r#"{
//             "status": "ok",
//             "retcode": 0,
//             "data": {
//                 "good": true,
//                 "online": true
//             },
//             "message": ""
//         }"#,
//         Resp::success(StatusContent {
//             good: true,
//             online: true,
//             extra: ExtendedMap::default(),
//         }),
//         Resp::success(RespContent::Status(StatusContent {
//             good: true,
//             online: true,
//             extra: ExtendedMap::default(),
//         })),
//     );
//     let empty_data = (
//         r#"{
//             "status": "ok",
//             "retcode": 0,
//             "data": {},
//             "message": ""
//         }"#,
//         Resp::success(HashMap::default()),
//         Resp::success(RespContent::Other(
//             HashMap::<_, ExtendedValue>::default().into(),
//         )),
//     );

//     assert_eq!(
//         serde_json::from_str::<Resp<StatusContent>>(status_data.0).unwrap(),
//         status_data.1
//     );
//     let json_str = serde_json::to_string(&status_data.1).unwrap();
//     assert_eq!(
//         serde_json::from_str::<Resp<RespContent<StandardEvent>>>(&json_str).unwrap(),
//         status_data.2
//     );

//     assert_eq!(
//         serde_json::from_str::<Resp<HashMap<String, ExtendedValue>>>(empty_data.0).unwrap(),
//         empty_data.1
//     );
//     let json_str = serde_json::to_string(&empty_data.1).unwrap();
//     assert_eq!(
//         serde_json::from_str::<Resp<RespContent<StandardEvent>>>(&json_str).unwrap(),
//         empty_data.2
//     );
// }

#[test]
fn message() {
    use crate::segment::*;
    use crate::prelude::WalleError;
    use crate::util::value::Value;
    use crate::{value_map, value};
    fn test<T>(message: (Value, MessageSegment, T))
    where
        T: TryFrom<MessageSegment, Error = WalleError> + std::fmt::Debug + PartialEq,
    {
        assert_eq!(MessageSegment::try_from(message.0).unwrap(), message.1);
        assert_eq!(T::try_from(message.1.clone()).unwrap(), message.2);
        println!("{}", message.1.alt());
    }

    test((
        value!({"type": "text",
            "data": {
                "text": "这是一个纯文本消息段"
            }
        }),
        MessageSegment {
            ty: "text".to_string(),
            data: value_map! {
                "text": "这是一个纯文本消息段"
            },
        },
        Text {
            text: "这是一个纯文本消息段".to_string(),
        },
    ));
    test((
        value!({"type": "image",
            "data": {
                "file_id": "e30f9684-3d54-4f65-b2da-db291a477f16",
                "url": "https://example.com"
            }
        }),
        MessageSegment {
            ty: "image".to_string(),
            data: value_map! {
                "file_id": "e30f9684-3d54-4f65-b2da-db291a477f16",
                "url": "https://example.com"
            },
        },
        BaseSegment {
            segment: Image {
                file_id: "e30f9684-3d54-4f65-b2da-db291a477f16".to_string(),
            },
            extra: value_map! {
                "url": "https://example.com"
            },
        },
    ));
}

#[test]
fn extendedmap_test() {
    use crate::util::{ValueMap, Value};
    let mut map = ValueMap::new();
    map.insert("key1".to_owned(), Value::Null);
    println!("{}", serde_json::to_string(&map).unwrap());
    let d = r#"{"key":3}"#;
    let map: ValueMap = serde_json::from_str(d).unwrap();
    println!("{:?}", map);
}

#[test]
fn enum_action() {
    use crate::action::*;
    use crate::prelude::value_map;
    use crate::prelude::WalleResult;
    use walle_macro::_OneBot as OneBot;
    #[derive(Debug, OneBot)]
    #[action]
    pub enum MyAction {
        GetUserInfo(GetUserInfo),
        GetGroupInfo { group_id: String },
    }

    let raw_action = Action {
        action: "get_user_info".to_string(),
        params: value_map! {
            "user_id": "abab"
        },
    };
    let action: WalleResult<BaseAction<MyAction>> = raw_action.try_into();
    println!("{:?}", action);
}

#[test]
fn option_action() {
    use crate::action::Action;
    use walle_macro::{_OneBot as OneBot, _PushToValueMap as PushToValueMap};
    #[derive(Debug, OneBot, PushToValueMap)]
    #[action]
    pub struct MySeg {
        pub text: Option<String>,
    }
    println!(
        "{:?}",
        MySeg::try_from(
            serde_json::from_str::<Action>(r#"{"action":"my_seg", "params": {"text": "text"}}"#)
                .unwrap()
        )
    );
    println!(
        "{:?}",
        MySeg::try_from(
            serde_json::from_str::<Action>(r#"{"action":"my_seg", "params": {}}"#).unwrap()
        )
    )
}
