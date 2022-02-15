use crate::message::Message;
use serde::{Deserialize, Serialize};
use walle_core::ExtendedMap;
mod resp;

pub use resp::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "params", rename_all = "snake_case")]
pub enum Action {
    SendPrivateMsg {
        user_id: i64,
        message: Message,
        auto_escape: bool,
    },
    SendGroupMsg {
        group_id: i64,
        message: Message,
        auto_escape: bool,
    },
    SendGroupForwardMsg {
        group_id: i64,
        message: Message,
    },
    SendMsg {
        message_type: String,
        user_id: Option<i64>,
        group_id: Option<i64>,
        message: Message,
        #[serde(default)]
        auto_escape: bool,
    },
    DeleteMsg {
        message_id: i32,
    },
    GetMsg {
        message_id: i32,
    },
    GetForwardMsg {
        message_id: String,
    },
    GetImage {
        file: String,
    },
    SetGroupKick {
        group_id: i64,
        user_id: i64,
        reject_add_request: bool,
    },
    SetGroupBan {
        group_id: i64,
        user_id: i64,
        duration: i64,
    },
    // SetGroupAnonymousBan {
    //     group_id: i64,
    //     flag: bool,
    // },
    SetGroupWholeBan {
        group_id: i64,
        enable: bool,
    },
    SetGroupAdmin {
        group_id: i64,
        user_id: i64,
        enable: bool,
    },
    SetGroupCard {
        group_id: i64,
        user_id: i64,
        card: String,
    },
    SetGroupName {
        group_id: i64,
        name: String,
    },
    SetGroupLeave {
        group_id: i64,
        is_dismiss: bool,
    },
    SetGroupSpecialTitle {
        group_id: i64,
        user_id: i64,
        special_title: String,
        duration: i64,
    },
    SetFriendAddRequest {
        flag: bool,
        approve: bool,
        remark: String,
    },
    SetGroupAddRequest {
        flag: bool,
        approve: bool,
        sub_type: String,
        reason: String,
    },
    GetLoginInfo(ExtendedMap),
    GetStrangerInfo {
        user_id: i64,
        no_cache: bool,
    },
    GetFriendList(ExtendedMap),
    DeleteFriend {
        friend_id: i64,
    },
    GetGroupInfo {
        group_id: i64,
        no_cache: bool,
    },
    GetGroupList(ExtendedMap),
    GetGroupMemberInfo {
        group_id: i64,
        user_id: i64,
        no_cache: bool,
    },
    GetGroupMemberList {
        group_id: i64,
    },
    GetVersionInfo(ExtendedMap),
    SendLike {
        user_id: i64,
        times: u8,
    },
}
