use crate::{
    message::MSVistor,
    resp::{
        FileIdContent, GroupInfoContent, SendMessageRespContent, StatusContent, UserInfoContent,
        VersionContent,
    },
    ExtendedMap, ExtendedValue, Message, Resp, WalleResult,
};
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
    GetMessage(GetMessage),

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

/// OneBot 12 扩展动作
///
/// 任何符合 OneBot 12 格式的动作均可序列化为该 struct
///
/// 如果需要使用该动作，可以使用 untagged enum 包裹该动作：
///
/// ```rust
/// use onebot_12::{StandardAction, ExtendedAction};
/// use serde::{Serialize, Deserialize};
///
/// #[derive(Serialize, Deserialize)]
/// #[serde(untagged)]
/// pub enum YourAction {
///     Standard(StandardAction),
///     Extended(ExtendedAction),
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ExtendedAction {
    pub action: String,
    pub params: ExtendedMap,
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
        MSVistor::_visit_map(map).map(|s| vec![s])
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GetLatestEvents {
    #[serde(default)]
    pub limit: i64,
    #[serde(default)]
    pub timeout: i64,
    #[serde(flatten)]
    pub extra: ExtendedMap,
}
// onebot_action!(GetLatestEvents, limit: i64, timeout: i64);
onebot_action!(DeleteMessage, message_id: String);
onebot_action!(GetMessage, message_id: String);
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
#[serde(tag = "stage", rename_all = "snake_case")]
pub enum UploadFileFragmented {
    Prepare {
        name: String,
        total_size: i64,
    },
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
        data: Vec<u8>,
    },
    Finish {
        file_id: String,
        sha256: String,
    },
}

/// Action content for GetFileFragmented
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "stage", rename_all = "snake_case")]
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
            GetMessage(walle_core::action::GetMessage),
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
                    walle_core::StandardAction::GetMessage(action) => $ext_name::GetMessage(action),
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

use std::{future::Future, pin::Pin};

type PBFR<'r, R> = Pin<Box<dyn Future<Output = WalleResult<R>> + Send + 'r>>;

macro_rules! exts {
    ($ex_name: ident, $name: ident, $rty: ty) => {
        fn $ex_name<'a, 'b>(
            &'a self,
            extra: ExtendedMap,
        ) -> PBFR<'b, $rty>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>;
        exts_noex!($ex_name, $name, $rty);
    };
    ($ex_name: ident, $name:ident, $rty: ty, $field_name: ident: $field_ty: ty) => {
        fn $ex_name<'a, 'b>(
            &'a self,
            $field_name: $field_ty,
            extra: ExtendedMap,
        ) -> PBFR<'b, $rty>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>;
        exts_noex!($ex_name, $name, $rty, $field_name: $field_ty);
    };
    ($ex_name: ident, $name:ident, $rty: ty, $($field_name: ident: $field_ty: ty),*) => {
        fn $ex_name<'a, 'b>(
            &'a self,
            $($field_name: $field_ty,)*
            extra: ExtendedMap,
        ) -> PBFR<'b, $rty>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>;
        exts_noex!($ex_name, $name, $rty, $($field_name: $field_ty),*);
    };
}

macro_rules! exts_noex {
    ($ex_name: ident, $name: ident, $rty: ty) => {
        fn $name<'a, 'b>(&'a self) -> PBFR<'b, $rty>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>,
        {
            self.$ex_name(ExtendedMap::default())
        }
    };
    ($ex_name: ident, $name:ident, $rty: ty, $field_name: ident: $field_ty: ty) => {
        fn $name<'a, 'b>(
            &'a self,
            $field_name: $field_ty,
        ) -> PBFR<'b, $rty>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>,
        {
            self.$ex_name($field_name, ExtendedMap::default())
        }
    };
    ($ex_name: ident, $name:ident, $rty: ty, $($field_name: ident: $field_ty: ty),*) => {
        fn $name<'a, 'b>(
            &'a self,
            $($field_name: $field_ty,)*
        ) -> PBFR<'b, $rty>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>,
        {
            self.$ex_name($($field_name,)* ExtendedMap::default())
        }
    };
}

pub trait GetLatestEventsExt<R, E>: Sync {
    exts!(
        get_latest_events_ex,
        get_latest_events,
        Resp<Vec<E>>,
        limit: i64,
        timeout: i64
    );
    exts!(get_message_ex, get_message, Resp<E>, message_id: String);
}

#[async_trait::async_trait]
pub trait BotActionExt<R>: Sync {
    exts!(
        get_supported_actions_ex,
        get_supported_actions,
        Resp<Vec<String>>
    );
    exts!(get_status_ex, get_status, Resp<StatusContent>);
    exts!(get_version_ex, get_version, Resp<VersionContent>);
    exts!(
        send_message_ex,
        send_message,
        Resp<SendMessageRespContent>,
        detail_type: String,
        group_id: Option<String>,
        user_id: Option<String>,
        message: Message
    );
    exts!(
        delete_message_ex,
        delete_message,
        Resp<ExtendedValue>,
        message_id: String
    );
    exts!(get_self_info_ex, get_self_info, Resp<UserInfoContent>);
    exts!(
        get_user_info_ex,
        get_user_info,
        Resp<UserInfoContent>,
        user_id: String
    );
    exts!(
        get_friend_list_ex,
        get_friend_list,
        Resp<Vec<UserInfoContent>>
    );
    exts!(
        get_group_info_ex,
        get_group_info,
        Resp<GroupInfoContent>,
        group_id: String
    );
    exts!(
        get_group_list_ex,
        get_group_list,
        Resp<Vec<GroupInfoContent>>
    );
    exts!(
        get_group_member_info_ex,
        get_group_member_info,
        Resp<UserInfoContent>,
        group_id: String,
        user_id: String
    );
    exts!(
        get_group_member_list_ex,
        get_group_member_list,
        Resp<Vec<GroupInfoContent>>,
        group_id: String
    );
    exts!(
        set_group_name_ex,
        set_group_name,
        Resp<ExtendedValue>,
        group_id: String,
        name: String
    );
    exts!(
        leave_group_ex,
        leave_group,
        Resp<ExtendedValue>,
        group_id: String
    );
    exts!(
        kick_group_member_ex,
        kick_group_member,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        ban_group_member_ex,
        ban_group_member,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        unban_group_member_ex,
        unban_group_member,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        set_group_admin_ex,
        set_group_admin,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        unset_group_admin_ex,
        unset_group_admin,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        upload_file_ex,
        upload_file,
        Resp<FileIdContent>,
        r#type: String,
        name: String,
        url: Option<String>,
        headers: Option<std::collections::HashMap<String, String>>,
        path: Option<String>,
        data: Option<Vec<u8>>,
        sha256: Option<String>
    );
    exts!(
        get_file_ex,
        get_file,
        Resp<UploadFile>,
        file_id: String,
        r#type: String
    );
    fn send_private_msg_ex<'a, 'b>(
        &'a self,
        user_id: String,
        message: Message,
        extra: ExtendedMap,
    ) -> PBFR<'b, Resp<SendMessageRespContent>>
    where
        'a: 'b,
        Self: 'b,
        Resp<SendMessageRespContent>: TryFrom<R>,
    {
        self.send_message_ex("private".to_string(), None, Some(user_id), message, extra)
    }
    exts_noex!(
        send_private_msg_ex,
        send_private_msg,
        Resp<SendMessageRespContent>,
        user_id: String,
        message: Message
    );
    fn send_group_msg_ex<'a, 'b>(
        &'a self,
        group_id: String,
        message: Message,
        extra: ExtendedMap,
    ) -> PBFR<'b, Resp<SendMessageRespContent>>
    where
        'a: 'b,
        Self: 'b,
        Resp<SendMessageRespContent>: TryFrom<R>,
    {
        self.send_message_ex("group".to_string(), Some(group_id), None, message, extra)
    }
    exts_noex!(
        send_group_msg_ex,
        send_group_msg,
        Resp<SendMessageRespContent>,
        group_id: String,
        message: Message
    );
    fn upload_file_by_url_ex<'a, 'b>(
        &'a self,
        name: String,
        url: String,
        headers: std::collections::HashMap<String, String>,
        sha256: Option<String>,
        extra: ExtendedMap,
    ) -> PBFR<'b, Resp<FileIdContent>>
    where
        'a: 'b,
        Self: 'b,
        Resp<FileIdContent>: TryFrom<R>,
    {
        self.upload_file_ex(
            "url".to_string(),
            name,
            Some(url),
            Some(headers),
            None,
            None,
            sha256,
            extra,
        )
    }
    exts_noex!(
        upload_file_by_url_ex,
        upload_file_by_url,
        Resp<FileIdContent>,
        name: String,
        url: String,
        headers: std::collections::HashMap<String, String>,
        sha256: Option<String>
    );
    fn upload_file_by_path_ex<'a, 'b>(
        &'a self,
        name: String,
        path: String,
        sha256: Option<String>,
        extra: ExtendedMap,
    ) -> PBFR<'b, Resp<FileIdContent>>
    where
        'a: 'b,
        Self: 'b,
        Resp<FileIdContent>: TryFrom<R>,
    {
        self.upload_file_ex(
            "path".to_string(),
            name,
            None,
            None,
            Some(path),
            None,
            sha256,
            extra,
        )
    }
    exts_noex!(
        upload_file_by_path_ex,
        upload_file_by_path,
        Resp<FileIdContent>,
        name: String,
        path: String,
        sha256: Option<String>
    );
    fn upload_file_by_data_ex<'a, 'b>(
        &'a self,
        name: String,
        data: Vec<u8>,
        sha256: Option<String>,
        extra: ExtendedMap,
    ) -> PBFR<'b, Resp<FileIdContent>>
    where
        'a: 'b,
        Self: 'b,
        Resp<FileIdContent>: TryFrom<R>,
    {
        self.upload_file_ex(
            "data".to_string(),
            name,
            None,
            None,
            None,
            Some(data),
            sha256,
            extra,
        )
    }
    exts_noex!(
        upload_file_by_data_ex,
        upload_file_by_data,
        Resp<FileIdContent>,
        name: String,
        data: Vec<u8>,
        sha256: Option<String>
    );
    async fn upload_file_fragmented(
        &self,
        name: String,
        file: tokio::fs::File,
    ) -> WalleResult<Resp<FileIdContent>>
    where
        Resp<FileIdContent>: TryFrom<R>;
}

pub trait ActionType {
    fn content_type(&self) -> crate::utils::ContentType;
}

impl ActionType for StandardAction {
    fn content_type(&self) -> crate::utils::ContentType {
        match self {
            Self::UploadFile(_)
            | Self::UploadFileFragmented(_)
            | Self::GetFile(_)
            | Self::GetFileFragmented(_) => crate::utils::ContentType::MsgPack,
            _ => crate::utils::ContentType::Json,
        }
    }
}
