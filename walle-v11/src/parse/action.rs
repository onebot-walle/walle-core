use super::message::MessageParseExt;
use crate::action::{Action as V11Action, Resp as V11Resp, RespContent as V11RespContent};
use walle_core::{
    action::{GroupIdContent, IdsContent, SendMessageContent, UserIdContent},
    Action as V12Action, EmptyContent, RespContent as V12RespContent, Resps as V12Resps,
};

impl TryFrom<V12Action> for V11Action {
    type Error = super::WalleParseError;
    fn try_from(value: V12Action) -> Result<Self, Self::Error> {
        match value {
            _ => todo!(),
        }
    }
}

impl TryInto<V12Action> for V11Action {
    type Error = super::WalleParseError;
    fn try_into(self) -> Result<V12Action, Self::Error> {
        match self {
            Self::SendMsg {
                message_type,
                user_id,
                group_id,
                message,
                ..
            } => Ok(V12Action::SendMessage(SendMessageContent {
                detail_type: message_type,
                user_id: user_id.map(|id| id.to_string()),
                group_id: group_id.map(|id| id.to_string()),
                message: message.try_parse()?,
            })),
            Self::SendPrivateMsg {
                user_id, message, ..
            } => Ok(V12Action::SendMessage(SendMessageContent {
                detail_type: "private".to_string(),
                user_id: Some(user_id.to_string()),
                group_id: None,
                message: message.try_parse()?,
            })),
            Self::SendGroupMsg {
                group_id, message, ..
            } => Ok(V12Action::SendMessage(SendMessageContent {
                detail_type: "group".to_string(),
                user_id: None,
                group_id: Some(group_id.to_string()),
                message: message.try_parse()?,
            })),
            Self::GetLoginInfo(_) => Ok(V12Action::GetSelfInfo(EmptyContent {})),
            Self::GetStrangerInfo { user_id, .. } => Ok(V12Action::GetUserInfo(UserIdContent {
                user_id: user_id.to_string(),
            })),
            Self::GetGroupInfo { group_id, .. } => Ok(V12Action::GetGroupInfo(GroupIdContent {
                group_id: group_id.to_string(),
            })),
            Self::GetFriendList(_) => Ok(V12Action::GetFriendList(EmptyContent {})),
            Self::GetGroupList(_) => Ok(V12Action::GetGroupList(EmptyContent {})),
            Self::GetGroupMemberList { group_id, .. } => {
                Ok(V12Action::GetGroupMemberList(GroupIdContent {
                    group_id: group_id.to_string(),
                }))
            }
            Self::GetGroupMemberInfo {
                group_id, user_id, ..
            } => Ok(V12Action::GetGroupMemberInfo(IdsContent {
                group_id: group_id.to_string(),
                user_id: user_id.to_string(),
                extended: [].into(),
            })),
            _ => todo!(),
        }
    }
}

impl From<V12Resps> for V11Resp {
    fn from(value: V12Resps) -> Self {
        Self {
            status: value.status,
            retcode: value.retcode,
            data: match value.data {
                V12RespContent::SendMessage(c) => V11RespContent::Message {
                    message_id: c.message_id.parse().unwrap(),
                },
                _ => todo!(),
            },
        }
    }
}

impl Into<V12Resps> for V11Resp {
    fn into(self) -> V12Resps {
        todo!();
    }
}
