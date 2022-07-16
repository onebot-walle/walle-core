use walle_core::event::{BaseEvent, Event};
use walle_core::extended_map;
use walle_core::segment::{Segments, MessageSegment};
use walle_core::prelude::OneBot;

#[derive(Debug, OneBot, PartialEq)]
#[event(type = "message")]
pub struct MessageE {
    pub message_id: String,
    pub message: Segments,
    pub alt_message: String,
    pub user_id: String,
}

#[derive(Debug, OneBot, PartialEq)]
#[event(detail_type = "private")]
pub struct Private {}

#[derive(Debug, OneBot, PartialEq)]
#[event(detail_type)]
pub struct Group {
    pub group_id: String,
}

fn main() {
    let raw_pme = Event {
        id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
        implt: "go_onebot_qq".to_owned(),
        platform: "qq".to_owned(),
        self_id: "123234".to_owned(),
        time: 1632847927.0,
        ty: "message".to_string(),
        detail_type: "private".to_string(),
        sub_type: "".to_string(),
        extra: extended_map! {
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
    };
    let pmbe = BaseEvent::<MessageE> {
        id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_string(),
        self_id: "123234".to_string(),
        time: 1632847927.0,
        implt: (),
        platform: (),
        ty: MessageE {
            message_id: "6283".to_string(),
            message: vec![
                MessageSegment {
                    ty: "text".to_string(),
                    data: extended_map! {"text": "OneBot is not a bot"},
                },
                MessageSegment {
                    ty: "image".to_string(),
                    data: extended_map! {"file_id": "e30f9684-3d54-4f65-b2da-db291a477f16"},
                },
            ],
            alt_message: "OneBot is not a bot[图片]".to_string(),
            user_id: "123456788".to_string(),
        },
        detail_type: (),
        sub_type: (),
        extra: extended_map!(),
    };
    let tpme: BaseEvent<MessageE> = raw_pme.clone().try_into().unwrap();
    assert_eq!(tpme, pmbe);
    let raw_gme = Event {
        id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_owned(),
        implt: "go_onebot_qq".to_owned(),
        platform: "qq".to_owned(),
        self_id: "123234".to_owned(),
        time: 1632847927.0,
        ty: "message".to_string(),
        detail_type: "group".to_string(),
        sub_type: "".to_string(),
        extra: extended_map! {
            "group_id": "group",
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
    };
    let gmbe = BaseEvent::<MessageE, Group> {
        id: "b6e65187-5ac0-489c-b431-53078e9d2bbb".to_string(),
        self_id: "123234".to_string(),
        time: 1632847927.0,
        implt: (),
        platform: (),
        ty: MessageE {
            message_id: "6283".to_string(),
            message: vec![
                MessageSegment {
                    ty: "text".to_string(),
                    data: extended_map! {"text": "OneBot is not a bot"},
                },
                MessageSegment {
                    ty: "image".to_string(),
                    data: extended_map! {"file_id": "e30f9684-3d54-4f65-b2da-db291a477f16"},
                },
            ],
            alt_message: "OneBot is not a bot[图片]".to_string(),
            user_id: "123456788".to_string(),
        },
        detail_type: Group {
            group_id: "group".to_string(),
        },
        sub_type: (),
        extra: extended_map!(),
    };
    let tgme: BaseEvent<MessageE, Group> = raw_gme.clone().try_into().unwrap();
    assert_eq!(tgme, gmbe);
}
