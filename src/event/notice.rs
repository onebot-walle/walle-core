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

impl super::EventType for NoticeContent {
    fn event_type(&self) -> &str {
        "notice"
    }
    fn detail_type(&self) -> &str {
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
    fn sub_type(&self) -> &str {
        match self {
            Self::GroupMemberIncrease { sub_type, .. } => sub_type,
            Self::GroupMemberDecrease { sub_type, .. } => sub_type,
            Self::GroupMemberBan { sub_type, .. } => sub_type,
            Self::GroupMemberUnban { sub_type, .. } => sub_type,
            Self::GroupAdminSet { sub_type, .. } => sub_type,
            Self::GroupAdminUnset { sub_type, .. } => sub_type,
            Self::GroupMessageDelete { sub_type, .. } => sub_type,
            Self::FriendIncrease { sub_type, .. } => sub_type,
            Self::FriendDecrease { sub_type, .. } => sub_type,
            Self::PrivateMessageDelete { sub_type, .. } => sub_type,
            Self::GuildMemberIncrease { sub_type, .. } => sub_type,
            Self::GuildMemberDecrease { sub_type, .. } => sub_type,
            Self::ChannelMessageDelete { sub_type, .. } => sub_type,
            Self::ChannelCreate { sub_type, .. } => sub_type,
            Self::ChannelDelete { sub_type, .. } => sub_type,
        }
    }
}
