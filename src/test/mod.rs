#[test]
fn event() {
    use crate::event::Event;
    use crate::event::{
        BaseEvent, EventContent, MessageContent as MsgContent, MessageEventType, MetaContent,
        NoticeContent,
    };
    use crate::resp::StatusContent;
    use crate::ExtendedMap;
    use crate::MessageSegment;
    use std::collections::HashMap;
    let data = vec![
        (
            r#"{
                "id": "b6e65187-5ac0-489c-b431-53078e9d2bbb",
                "impl": "go_onebot_qq",
                "platform": "qq",
                "self_id": "123234",
                "time": 1632847927,
                "type": "meta",
                "detail_type": "heartbeat",
                "sub_type": "",
                "interval": 5000,
                "status": {
                    "good": true,
                    "online": true
                }
            }"#,
            BaseEvent {
                id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
                r#impl: "go_onebot_qq".to_owned(),
                platform: "qq".to_owned(),
                self_id: "123234".to_owned(),
                time: 1632847927.0,
                content: EventContent::Meta(MetaContent::Heartbeat {
                    interval: 5000,
                    status: StatusContent {
                        good: true,
                        online: true,
                    },
                    sub_type: "".to_owned(),
                }),
            },
        ),
        (
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
            BaseEvent {
                id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
                r#impl: "go_onebot_qq".to_owned(),
                platform: "qq".to_owned(),
                self_id: "123234".to_owned(),
                time: 1632847927.0,
                content: EventContent::Message(MsgContent {
                    sub_type: String::default(),
                    ty: MessageEventType::Private,
                    message_id: "6283".to_owned(),
                    message: vec![
                        MessageSegment::Text {
                            text: "OneBot is not a bot".to_owned(),
                            extend: HashMap::new(),
                        },
                        MessageSegment::Image {
                            file_id: "e30f9684-3d54-4f65-b2da-db291a477f16".to_owned(),
                            extend: HashMap::new(),
                        },
                    ],
                    alt_message: "OneBot is not a bot[图片]".to_owned(),
                    user_id: "123456788".to_owned(),
                    extra: ExtendedMap::default(),
                }),
            },
        ),
        (
            r#"{
                "id": "b6e65187-5ac0-489c-b431-53078e9d2bbb",
                "impl": "go_onebot_qq",
                "platform": "qq",
                "self_id": "123234",
                "time": 1632847927,
                "type": "notice",
                "detail_type": "group_member_increase",
                "sub_type": "join",
                "user_id": "123456788",
                "group_id": "87654321",
                "operator_id": "1234567"
            }"#,
            BaseEvent {
                id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
                r#impl: "go_onebot_qq".to_owned(),
                platform: "qq".to_owned(),
                self_id: "123234".to_owned(),
                time: 1632847927.0,
                content: EventContent::Notice(NoticeContent::GroupMemberIncrease {
                    sub_type: "join".to_owned(),
                    group_id: "87654321".to_owned(),
                    user_id: "123456788".to_owned(),
                    operator_id: "1234567".to_owned(),
                }),
            },
        ),
    ];

    for (json, event) in data {
        assert_eq!(serde_json::from_str::<Event>(json).unwrap(), event);
        let json_str = serde_json::to_string(&event).unwrap();
        assert_eq!(serde_json::from_str::<Event>(&json_str).unwrap(), event);
    }
}

#[test]
fn action() {
    use crate::action::GetLatestEvents;
    use crate::action::*;
    use crate::Echo;
    use crate::{StandardAction, MessageSegment};
    use std::collections::HashMap;

    let data = vec![
        (
            r#"{
            "action": "get_latest_events",
            "params": {
                "limit": 100,
                "timeout": 0
            }
        }"#,
            StandardAction::GetLatestEvents(GetLatestEvents {
                limit: 100,
                timeout: 0,
                extra: [].into(),
            }),
        ),
        (
            r#"{
                "action": "send_message",
                "params": {
                    "detail_type": "group",
                    "group_id": "12467",
                    "message": "我是文字巴拉巴拉巴拉"
                }
            }"#,
            StandardAction::SendMessage(SendMessage {
                detail_type: "group".to_owned(),
                group_id: Some("12467".to_owned()),
                user_id: None,
                message: vec![MessageSegment::Text {
                    text: "我是文字巴拉巴拉巴拉".to_owned(),
                    extend: HashMap::new(),
                }],
                extra: [].into(),
            }),
        ),
        (
            r#"{
                "action": "get_self_info",
                "params": {}
            }"#,
            StandardAction::GetSelfInfo([].into()),
        ),
    ];

    for (json, action) in data {
        assert_eq!(
            serde_json::from_str::<Echo<StandardAction>>(json)
                .unwrap()
                .unpack()
                .0,
            action
        );
        let json_str = serde_json::to_string(&action).unwrap();
        assert_eq!(serde_json::from_str::<StandardAction>(&json_str).unwrap(), action);
    }
}

#[test]
fn action_resp() {
    use crate::resp::*;
    use crate::ExtendedValue;
    use std::collections::HashMap;

    let status_data = (
        r#"{   
            "status": "ok",
            "retcode": 0,
            "data": {
                "good": true,
                "online": true
            },
            "message": ""
        }"#,
        Resp::success(StatusContent {
            good: true,
            online: true,
        }),
        Resp::success(RespContent::Status(StatusContent {
            good: true,
            online: true,
        })),
    );
    let empty_data = (
        r#"{
            "status": "ok",
            "retcode": 0,
            "data": {},
            "message": ""
        }"#,
        Resp::success(HashMap::default()),
        Resp::success(RespContent::Other(
            HashMap::<_, ExtendedValue>::default().into(),
        )),
    );

    assert_eq!(
        serde_json::from_str::<Resp<StatusContent>>(status_data.0).unwrap(),
        status_data.1
    );
    let json_str = serde_json::to_string(&status_data.1).unwrap();
    assert_eq!(
        serde_json::from_str::<Resp<RespContent>>(&json_str).unwrap(),
        status_data.2
    );

    assert_eq!(
        serde_json::from_str::<Resp<HashMap<String, ExtendedValue>>>(empty_data.0).unwrap(),
        empty_data.1
    );
    let json_str = serde_json::to_string(&empty_data.1).unwrap();
    assert_eq!(
        serde_json::from_str::<Resp<RespContent>>(&json_str).unwrap(),
        empty_data.2
    );
}

#[test]
fn message() {
    use crate::message::{Message, MessageBuild, MessageSegment};
    let message = r#"{
        "type": "ctext",
        "data": {
            "text": "这是一个纯文本消息段"
        }
    }"#;
    let location_message = r#"{
        "type": "location",
        "data": {
            "latitude": 31.032315,
            "longitude": 121.447127,
            "title": "上海交通大学闵行校区",
            "content": "中国上海市闵行区东川路800号"
        }
    }"#;
    let text: MessageSegment = serde_json::from_str(message).unwrap();
    let loc: MessageSegment = serde_json::from_str(location_message).unwrap();
    let location = Message::new().location(1.1, 2.2, "title".to_owned(), "content".to_owned());
    println!("{:?}\n{:?}", text, loc);
    println!("{}", serde_json::to_string(&location).unwrap())
}

#[test]
fn extendedmap_test() {
    use crate::{ExtendedMap, ExtendedValue};
    let mut map = ExtendedMap::new();
    map.insert("key1".to_owned(), ExtendedValue::Null);
    println!("{}", serde_json::to_string(&map).unwrap());
    let d = r#"{"key":3}"#;
    let map: ExtendedMap = serde_json::from_str(d).unwrap();
    println!("{:?}", map);
}
