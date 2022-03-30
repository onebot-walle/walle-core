use crate::{message::MSVister, ExtendedMap};
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

/// ## OneBot 12 标准动作
///
/// **动作请求**是应用端为了主动向 OneBot 实现请求服务而发送的数据。
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "action", content = "params", rename_all = "snake_case")]
pub enum StandardAction {
    // meta action
    GetLatestEvents(GetLatestEvents),
    GetSupportedActions(ExtendedMap),
    GetStatus(ExtendedMap),
    GetVersion(ExtendedMap),

    // message action
    SendMessage(SendMessage),
    DeleteMessage(DeleteMessage),

    // user action
    GetSelfInfo(ExtendedMap),
    GetUserInfo(GetUserInfo),
    GetFriendList(ExtendedMap),

    // group action
    GetGroupInfo(GetGroupInfo),
    GetGroupList(ExtendedMap),
    GetGroupMemberInfo(GetGroupMemberInfo),
    GetGroupMemberList(GetGroupMemberList),
    SetGroupName(SetGroupName),
    LeaveGroup(LeaveGroup),
    KickGroupMember(KickGroupMember),
    BanGroupMember(BanGroupMember),
    UnbanGroupMember(UnbanGroupMember),
    SetGroupAdmin(SetGroupAdmin),
    UnsetGroupAdmin(UnsetGroupAdmin),

    // file
    UploadFile(UploadFile),
    UploadFileFragmented(UploadFileFragmented),
    GetFile(GetFile),
    GetFileFragmented(GetFileFragmented),
}

macro_rules! impl_from(
    ($to:ident => $($sub: ident),*) => {
        $(impl From<$sub> for $to {
            fn from(from: $sub) -> Self {
                $to::$sub(from)
            }
        })*
    };
);

/// Action content for SendMessage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SendMessage {
    pub detail_type: String,
    pub group_id: Option<String>,
    pub user_id: Option<String>,
    #[serde(deserialize_with = "deserialize_message")]
    pub message: crate::Message,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}

struct MessageVisitor;

impl<'de> Visitor<'de> for MessageVisitor {
    type Value = crate::Message;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a message")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::SeqAccess<'de>,
    {
        let mut message = Vec::new();
        while let Some(segment) = seq.next_element()? {
            message.push(segment);
        }
        Ok(message)
    }

    fn visit_map<A>(self, map: A) -> Result<Self::Value, A::Error>
    where
        A: serde::de::MapAccess<'de>,
    {
        MSVister::_visit_map(map).map(|s| vec![s])
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(vec![crate::MessageSegment::text(s.to_owned())])
    }
}

fn deserialize_message<'de, D>(deserializer: D) -> Result<crate::Message, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_any(MessageVisitor)
}

#[macro_export]
macro_rules! onebot_action {
    ($action_name: ident, $($field_name: ident: $field_ty: ty),*) => {
        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
        pub struct $action_name {
            $(pub $field_name: $field_ty,)*
            #[serde(flatten)]
            pub extra: ExtendedMap,
        }
    };
}

// #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
// pub struct GetLatestEvents {
//     pub limit: i64,
//     pub timeout: i64,
// }
onebot_action!(GetLatestEvents, limit: i64, timeout: i64);
onebot_action!(DeleteMessage, message_id: String);
onebot_action!(GetUserInfo, user_id: String);
onebot_action!(GetGroupInfo, group_id: String);
onebot_action!(GetGroupMemberList, group_id: String);
onebot_action!(LeaveGroup, group_id: String);
onebot_action!(GetGroupMemberInfo, group_id: String, user_id: String);
onebot_action!(KickGroupMember, group_id: String, user_id: String);
onebot_action!(BanGroupMember, group_id: String, user_id: String);
onebot_action!(UnbanGroupMember, group_id: String, user_id: String);
onebot_action!(SetGroupAdmin, group_id: String, user_id: String);
onebot_action!(UnsetGroupAdmin, group_id: String, user_id: String);
onebot_action!(SetGroupName, group_id: String, group_name: String);
onebot_action!(
    UploadFile,
    r#type: String,
    name: String,
    url: Option<String>,
    headers: Option<std::collections::HashMap<String, String>>,
    path: Option<String>,
    data: Option<Vec<u8>>,
    sha256: Option<String>
);
onebot_action!(GetFile, file_id: String, r#type: String);

/// Action content for UploadFileFragmented
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "stage")]
#[serde(rename_all = "snake_case")]
pub enum UploadFileFragmented {
    Prepare {
        name: String,
        total: i64,
        sha256: String,
    },
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
        data: Vec<u8>,
    },
    Finish {
        file_id: String,
    },
}

/// Action content for GetFileFragmented
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "stage")]
#[serde(rename_all = "snake_case")]
pub enum GetFileFragmented {
    Prepare {
        file_id: String,
    },
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
    },
}

impl_from!(
    StandardAction =>
    SendMessage,
    GetLatestEvents,
    DeleteMessage,
    GetUserInfo,
    GetGroupInfo,
    GetGroupMemberList,
    LeaveGroup,
    GetGroupMemberInfo,
    KickGroupMember,
    BanGroupMember,
    UnbanGroupMember,
    SetGroupAdmin,
    UnsetGroupAdmin,
    SetGroupName,
    UploadFile,
    UploadFileFragmented,
    GetFile,
    GetFileFragmented
);

#[macro_export]
macro_rules! onebot_action_ext {
    ($ext_name: ident => $($action: ident),*) => {
        #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
        #[serde(tag = "action", content = "params", rename_all = "snake_case")]
        pub enum $ext_name {
            GetLatestEvents(walle_core::action::GetLatestEvents),
            GetSupportedActions(walle_core::ExtendedMap),
            GetStatus(walle_core::ExtendedMap),
            GetVersion(walle_core::ExtendedMap),
            SendMessage(walle_core::action::SendMessage),
            DeleteMessage(walle_core::action::DeleteMessage),
            GetSelfInfo(walle_core::ExtendedMap),
            GetUserInfo(walle_core::action::GetUserInfo),
            GetFriendList(walle_core::ExtendedMap),
            GetGroupInfo(walle_core::action::GetGroupInfo),
            GetGroupList(walle_core::ExtendedMap),
            GetGroupMemberInfo(walle_core::action::GetGroupMemberInfo),
            GetGroupMemberList(walle_core::action::GetGroupMemberList),
            SetGroupName(walle_core::action::SetGroupName),
            LeaveGroup(walle_core::action::LeaveGroup),
            KickGroupMember(walle_core::action::KickGroupMember),
            BanGroupMember(walle_core::action::BanGroupMember),
            UnbanGroupMember(walle_core::action::UnbanGroupMember),
            SetGroupAdmin(walle_core::action::SetGroupAdmin),
            UnsetGroupAdmin(walle_core::action::UnsetGroupAdmin),
            UploadFile(walle_core::action::UploadFile),
            UploadFileFragmented(walle_core::action::UploadFileFragmented),
            GetFile(walle_core::action::GetFile),
            GetFileFragmented(walle_core::action::GetFileFragmented),
            $($action($action)),*
        }

        impl From<walle_core::StandardAction> for $ext_name {
            fn from(from: walle_core::StandardAction) -> $ext_name {
                match from {
                    walle_core::StandardAction::GetLatestEvents(action) => $ext_name::GetLatestEvents(action),
                    walle_core::StandardAction::GetSupportedActions(action) => $ext_name::GetSupportedActions(action),
                    walle_core::StandardAction::GetStatus(action) => $ext_name::GetStatus(action),
                    walle_core::StandardAction::GetVersion(action) => $ext_name::GetVersion(action),
                    walle_core::StandardAction::SendMessage(action) => $ext_name::SendMessage(action),
                    walle_core::StandardAction::DeleteMessage(action) => $ext_name::DeleteMessage(action),
                    walle_core::StandardAction::GetSelfInfo(action) => $ext_name::GetSelfInfo(action),
                    walle_core::StandardAction::GetUserInfo(action) => $ext_name::GetUserInfo(action),
                    walle_core::StandardAction::GetFriendList(action) => $ext_name::GetFriendList(action),
                    walle_core::StandardAction::GetGroupInfo(action) => $ext_name::GetGroupInfo(action),
                    walle_core::StandardAction::GetGroupList(action) => $ext_name::GetGroupList(action),
                    walle_core::StandardAction::GetGroupMemberInfo(action) => $ext_name::GetGroupMemberInfo(action),
                    walle_core::StandardAction::GetGroupMemberList(action) => $ext_name::GetGroupMemberList(action),
                    walle_core::StandardAction::SetGroupName(action) => $ext_name::SetGroupName(action),
                    walle_core::StandardAction::LeaveGroup(action) => $ext_name::LeaveGroup(action),
                    walle_core::StandardAction::KickGroupMember(action) => $ext_name::KickGroupMember(action),
                    walle_core::StandardAction::BanGroupMember(action) => $ext_name::BanGroupMember(action),
                    walle_core::StandardAction::UnbanGroupMember(action) => $ext_name::UnbanGroupMember(action),
                    walle_core::StandardAction::SetGroupAdmin(action) => $ext_name::SetGroupAdmin(action),
                    walle_core::StandardAction::UnsetGroupAdmin(action) => $ext_name::UnsetGroupAdmin(action),
                    walle_core::StandardAction::UploadFile(action) => $ext_name::UploadFile(action),
                    walle_core::StandardAction::UploadFileFragmented(action) => $ext_name::UploadFileFragmented(action),
                    walle_core::StandardAction::GetFile(action) => $ext_name::GetFile(action),
                    walle_core::StandardAction::GetFileFragmented(action) => $ext_name::GetFileFragmented(action),
                }
            }
        }

        $(impl From<$action> for $ext_name {
            fn from(from: $action) -> $ext_name {
                $ext_name::$action(from)
            }
        })*
    };
}
