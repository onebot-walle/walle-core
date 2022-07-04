use crate::util::ExtendedMap;
use serde::{Deserialize, Serialize};
use snake_cased::SnakedEnum;

/// ## OneBot 通知事件 Content
///
/// 通知事件是机器人平台向机器人发送通知对应的事件，例如群成员变动等。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, SnakedEnum)]
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

impl super::EventSubType for NoticeContent {
    fn sub_type(&self) -> &str {
        match self {
            NoticeContent::GroupMemberIncrease { sub_type, .. } => sub_type,
            NoticeContent::GroupMemberDecrease { sub_type, .. } => sub_type,
            NoticeContent::GroupMessageDelete { sub_type, .. } => sub_type,
            NoticeContent::FriendIncrease { sub_type, .. } => sub_type,
            NoticeContent::FriendDecrease { sub_type, .. } => sub_type,
            NoticeContent::PrivateMessageDelete { sub_type, .. } => sub_type,
            NoticeContent::GuildMemberIncrease { sub_type, .. } => sub_type,
            NoticeContent::GuildMemberDecrease { sub_type, .. } => sub_type,
            NoticeContent::ChannelMessageDelete { sub_type, .. } => sub_type,
            NoticeContent::ChannelCreate { sub_type, .. } => sub_type,
            NoticeContent::ChannelDelete { sub_type, .. } => sub_type,
        }
    }
}

impl super::EventDetailType for NoticeContent {
    fn detail_type(&self) -> &str {
        self.snaked_enum()
    }
}

impl super::EventType for NoticeContent {
    fn ty(&self) -> &str {
        "notice"
    }
}
