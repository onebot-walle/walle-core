use crate::ExtendedMap;
use serde::{Deserialize, Serialize};

/// ## OneBot 通知事件 Content
///
/// 通知事件是机器人平台向机器人发送通知对应的事件，例如群成员变动等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "detail_type", rename_all = "snake_case")]
pub enum NoticeContent {
    GroupMemberIncrease {
        sub_type: String,
        group_id: String,
        user_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    GroupMemberDecrease {
        sub_type: String,
        group_id: String,
        user_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    GroupMemberBan {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    GroupMemberUnban {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    GroupAdminSet {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    GroupAdminUnset {
        sub_type: String, // just for Deserialize
        group_id: String,
        user_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    GroupMessageDelete {
        sub_type: String,
        group_id: String,
        message_id: String,
        user_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    FriendIncrease {
        sub_type: String, // just for Deserialize
        user_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    FriendDecrease {
        sub_type: String, // just for Deserialize
        user_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    PrivateMessageDelete {
        sub_type: String, // just for Deserialize
        message_id: String,
        user_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    GuildMemberIncrease {
        sub_type: String,
        guild_id: String,
        user_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    GuildMemberDecrease {
        sub_type: String,
        guild_id: String,
        user_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    ChannelMessageDelete {
        sub_type: String,
        guild_id: String,
        channel_id: String,
        user_id: String,
        operator_id: String,
        message_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    ChannelCreate {
        sub_type: String,
        guild_id: String,
        channel_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
    },
    ChannelDelete {
        sub_type: String,
        guild_id: String,
        channel_id: String,
        operator_id: String,
        #[serde(flatten)]
        extra: ExtendedMap,
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
            Self::GuildMemberIncrease { .. } => "guild_member_increase",
            Self::GuildMemberDecrease { .. } => "guild_member_decrease",
            Self::ChannelMessageDelete { .. } => "channel_message_delete",
            Self::ChannelCreate { .. } => "channel_create",
            Self::ChannelDelete { .. } => "channel_delete",
        }
    }
}
