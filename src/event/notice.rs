use serde::{Deserialize, Serialize};

/// ## OneBot 通知事件 Content
///
/// 通知事件是机器人平台向机器人发送通知对应的事件，例如群成员变动等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "detail_type")]
#[serde(rename_all = "snake_case")]
pub enum NoticeContent {
    GroupMemberIncrease {
        sub_type: String,
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupMemberDecrease {
        sub_type: String,
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupMemberBan {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupMemberUnban {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupAdminSet {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupAdminUnset {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
    },
    GroupMessageDelete {
        sub_type: String,
        group_id: String,
        message_id: String,
        user_id: String,
        operator_id: String,
    },
    FriendIncrease {
        sub_type: String, // just for Deserialize
        user_id: String,
    },
    FriendDecrease {
        sub_type: String, // just for Deserialize
        user_id: String,
    },
    PrivateMessageDelete {
        sub_type: String, // just for Deserialize
        message_id: String,
        user_id: String,
    },
}

impl NoticeContent {
    pub fn detail_type(&self) -> &'static str {
        match self {
            Self::GroupMemberIncrease { .. } => "group_member_increase",
            Self::GroupMemberDecrease { .. } => "group_member_decrease",
            Self::GroupMemberBan { .. } => "group_member_ban",
            Self::GroupMemberUnban { .. } => "group_member_unban",
            Self::GroupAdminSet { .. } => "group_admin_set",
            Self::GroupAdminUnset { .. } => "group_admin_unset",
            Self::GroupMessageDelete { .. } => "group_message_delete",
            Self::FriendIncrease { .. } => "friend_increase",
            Self::FriendDecrease { .. } => "friend_decrease",
            Self::PrivateMessageDelete { .. } => "private_message_delete",
        }
    }
}
