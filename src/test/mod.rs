#[test]
fn event() {
    use crate::action_resp::StatusContent;
    use crate::event::Events;
    use crate::event::{Event, EventContent, Message as MsgContent, Meta, Notice};
    use crate::MessageSegment;

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
            Event {
                id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
                r#impl: "go_onebot_qq".to_owned(),
                platform: "qq".to_owned(),
                self_id: "123234".to_owned(),
                time: 1632847927,
                content: EventContent::Meta(Meta::Heartbeat {
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
            Event {
                id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
                r#impl: "go_onebot_qq".to_owned(),
                platform: "qq".to_owned(),
                self_id: "123234".to_owned(),
                time: 1632847927,
                content: EventContent::Message(MsgContent {
                    sub_type: String::default(),
                    ty: "private".to_owned(),
                    message_id: "6283".to_owned(),
                    message: vec![
                        MessageSegment::Text {
                            text: "OneBot is not a bot".to_owned(),
                        },
                        MessageSegment::Image {
                            file_id: "e30f9684-3d54-4f65-b2da-db291a477f16".to_owned(),
                        },
                    ],
                    alt_message: "OneBot is not a bot[图片]".to_owned(),
                    user_id: "123456788".to_owned(),
                    group_id: None,
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
            Event {
                id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
                r#impl: "go_onebot_qq".to_owned(),
                platform: "qq".to_owned(),
                self_id: "123234".to_owned(),
                time: 1632847927,
                content: EventContent::Notice(Notice::GroupMemberIncrease {
                    sub_type: "join".to_owned(),
                    group_id: "87654321".to_owned(),
                    user_id: "123456788".to_owned(),
                    operator_id: "1234567".to_owned(),
                }),
            },
        ),
    ];

    for (json, event) in data {
        assert_eq!(serde_json::from_str::<Events>(json).unwrap(), event);
        let json_str = serde_json::to_string(&event).unwrap();
        assert_eq!(serde_json::from_str::<Events>(&json_str).unwrap(), event);
    }
}

#[test]
fn action() {
    use crate::action::GetLatestEventsContent;
    use crate::action::*;
    use crate::{Action, MessageSegment};

    let data = vec![
        (
            r#"{
            "action": "get_latest_events",
            "params": {
                "limit": 100,
                "timeout": 0
            }
        }"#,
            Action::GetLatestEvents(GetLatestEventsContent {
                limit: 100,
                timeout: 0,
            }),
        ),
        (
            r#"{
                "action": "send_message",
                "params": {
                    "detail_type": "group",
                    "group_id": "12467",
                    "message": [
                        {
                            "type": "text",
                            "data": {
                                "text": "我是文字巴拉巴拉巴拉"
                            }
                        }
                    ]
                }
            }"#,
            Action::SendMessage(SendMessageContent {
                detail_type: "group".to_owned(),
                group_id: Some("12467".to_owned()),
                user_id: None,
                message: vec![MessageSegment::Text {
                    text: "我是文字巴拉巴拉巴拉".to_owned(),
                }],
            }),
        ),
        (
            r#"{
                "action": "get_self_info",
                "params": {}
            }"#,
            Action::GetSelfInfo(EmptyContent::default()),
        ),
    ];

    for (json, action) in data {
        assert_eq!(serde_json::from_str::<Action>(json).unwrap(), action);
        let json_str = serde_json::to_string(&action).unwrap();
        assert_eq!(serde_json::from_str::<Action>(&json_str).unwrap(), action);
    }
}

#[test]
fn action_resp() {
    use crate::action::EmptyContent;
    use crate::action_resp::*;

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
        ActionResp::success(StatusContent {
            good: true,
            online: true,
        }),
        ActionResp::success(ActionRespContent::Status(StatusContent {
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
        ActionResp::success(EmptyContent::default()),
        ActionResp::success(ActionRespContent::Empty(EmptyContent::default())),
    );

    assert_eq!(
        serde_json::from_str::<ActionResp<StatusContent>>(status_data.0).unwrap(),
        status_data.1
    );
    let json_str = serde_json::to_string(&status_data.1).unwrap();
    assert_eq!(
        serde_json::from_str::<ActionResp<ActionRespContent>>(&json_str).unwrap(),
        status_data.2
    );

    assert_eq!(
        serde_json::from_str::<ActionResp<EmptyContent>>(empty_data.0).unwrap(),
        empty_data.1
    );
    let json_str = serde_json::to_string(&empty_data.1).unwrap();
    assert_eq!(
        serde_json::from_str::<ActionResp<ActionRespContent>>(&json_str).unwrap(),
        empty_data.2
    );
}
