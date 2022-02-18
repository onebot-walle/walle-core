use super::message::MessageParseExt;
use crate::action::{Action as V11Action, Resp as V11Resp, RespContent as V11RespContent};
use walle_core::{
    action::{DeleteMessageContent, GroupIdContent, IdsContent, SendMessageContent, UserIdContent},
    Action as V12Action, ExtendedMap, RespContent as V12RespContent, Resps as V12Resps,
};

impl TryFrom<V12Action> for V11Action {
    type Error = super::WalleParseError;
    fn try_from(value: V12Action) -> Result<Self, Self::Error> {
        match value {
            _ => todo!(),
        }
    }
}

impl TryFrom<V11Action> for V12Action {
    type Error = super::WalleParseError;
    fn try_from(action: V11Action) -> Result<Self, Self::Error> {
        match action {
            V11Action::SendMsg {
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
            V11Action::SendPrivateMsg {
                user_id, message, ..
            } => Ok(V12Action::SendMessage(SendMessageContent {
                detail_type: "private".to_string(),
                user_id: Some(user_id.to_string()),
                group_id: None,
                message: message.try_parse()?,
            })),
            V11Action::SendGroupMsg {
                group_id, message, ..
            } => Ok(V12Action::SendMessage(SendMessageContent {
                detail_type: "group".to_string(),
                user_id: None,
                group_id: Some(group_id.to_string()),
                message: message.try_parse()?,
            })),
            V11Action::DeleteMsg { message_id, .. } => {
                Ok(V12Action::DeleteMessage(DeleteMessageContent {
                    message_id: message_id.to_string(),
                }))
            }
            V11Action::GetLoginInfo(_) => Ok(V12Action::GetSelfInfo(ExtendedMap::default())),
            V11Action::GetStrangerInfo { user_id, .. } => {
                Ok(V12Action::GetUserInfo(UserIdContent {
                    user_id: user_id.to_string(),
                }))
            }
            V11Action::GetGroupInfo { group_id, .. } => {
                Ok(V12Action::GetGroupInfo(GroupIdContent {
                    group_id: group_id.to_string(),
                }))
            }
            V11Action::GetFriendList(_) => Ok(V12Action::GetFriendList(ExtendedMap::default())),
            V11Action::GetGroupList(_) => Ok(V12Action::GetGroupList(ExtendedMap::default())),
            V11Action::GetGroupMemberList { group_id, .. } => {
                Ok(V12Action::GetGroupMemberList(GroupIdContent {
                    group_id: group_id.to_string(),
                }))
            }
            V11Action::GetGroupMemberInfo {
                group_id, user_id, ..
            } => Ok(V12Action::GetGroupMemberInfo(IdsContent {
                group_id: group_id.to_string(),
                user_id: user_id.to_string(),
                extra: [].into(),
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
                V12RespContent::Other(m) => V11RespContent::Other(m),
                V12RespContent::UserInfo(c) => V11RespContent::UserInfo {
                    user_id: c.user_id.parse().unwrap(),
                    nickname: c.nickname,
                    sex: "".to_owned(),
                    age: 0,
                },
                _ => todo!(),
            },
        }
    }
}

impl From<V11Resp> for V12Resps {
    fn from(_resp: V11Resp) -> Self {
        todo!();
    }
}
