#[test]
fn events() {
    use crate::action_resp::StatusContent;
    use crate::event::Events;
    use crate::event::{Event, EventContent, Message as MsgContent, Meta, Notice};
    use crate::MessageSegment;

    static META_EVENT_DATA: &str = r#"{
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
    }"#;

    let meta_event = Event {
        id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
        r#impl: "go_onebot_qq".to_owned(),
        platform: "qq".to_owned(),
        self_id: "123234".to_owned(),
        time: 1632847927,
        content: EventContent::Meta(Meta::HeartBeat {
            interval: 5000,
            status: StatusContent {
                good: true,
                online: true,
            },
            sub_type: "".to_owned(),
        }),
    };

    assert_eq!(
        serde_json::from_str::<Events>(META_EVENT_DATA).unwrap(),
        meta_event
    );
    let json_str = serde_json::to_string(&meta_event).unwrap();
    assert_eq!(
        serde_json::from_str::<Events>(&json_str).unwrap(),
        meta_event
    );


    static MESSAGE_EVENT_DATA: &str = r#"{
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
    }"#;

    let message_event = Event {
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
    };

    assert_eq!(
        serde_json::from_str::<Events>(MESSAGE_EVENT_DATA).unwrap(),
        message_event
    );
    let json_str = serde_json::to_string(&message_event).unwrap();
    assert_eq!(
        serde_json::from_str::<Events>(&json_str).unwrap(),
        message_event
    );

    static NOTICE_EVENT_DATA: &str = r#"{
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
    }"#;

    let notice_event = Event {
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
    };

    assert_eq!(
        serde_json::from_str::<Events>(NOTICE_EVENT_DATA).unwrap(),
        notice_event
    );
    let json_str = serde_json::to_string(&notice_event).unwrap();
    assert_eq!(
        serde_json::from_str::<Events>(&json_str).unwrap(),
        notice_event
    );
}
