use crate::{action::*, ActionResp, EmptyContent};

macro_rules! action_api {
    ($fn_name: ident,$action_ty:ident,$content:ident) => {
        pub async fn $fn_name(&self) -> Option<ActionResp<R>> {
            self.call_action_resp(Action::$action_ty($content::default())).await
        }
    };
    ($fn_name: ident,$action_ty:ident,$content:ident, $field_name: ident: $field_ty: ty) => {
        pub async fn $fn_name(&self, $field_name: $field_ty) -> Option<ActionResp<R>> {
            self.call_action_resp(Action::$action_ty($content{
                $field_name,
            })).await
        }
    };
    ($fn_name: ident,$action_ty:ident,$content:ident, $($field_name: ident: $field_ty: ty),*) => {
        pub async fn $fn_name(&self, $($field_name: $field_ty,)*) -> Option<ActionResp<R>> {
            self.call_action_resp(Action::$action_ty($content{
                $($field_name,)*
            })).await
        }
    };
}

impl<E, R> super::CustomOneBot<E, Action, R>
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    pub async fn get_latest_events(&self, limit: i64, timeout: i64) -> Option<ActionResp<R>> {
        self.call_action_resp(Action::GetLatestEvents(GetLatestEventsContent {
            limit,
            timeout,
        }))
        .await
    }
    action_api!(get_support_actions, GetSupportedActions, EmptyContent);
    action_api!(get_status, GetStatus, EmptyContent);
    action_api!(get_version, GetVersion, EmptyContent);

    action_api!(
        send_message,
        SendMessage,
        SendMessageContent,
        detail_type: String,
        group_id: Option<String>,
        user_id: Option<String>,
        message: crate::Message
    );
    action_api!(
        deletemessage,
        DeleteMessage,
        DeleteMessageContent,
        message_id: String
    );

    action_api!(get_self_info, GetSelfInfo, EmptyContent);
    action_api!(get_user_info, GetUserInfo, UserIdContent, user_id: String);
    action_api!(get_friend_list, GetFriendList, EmptyContent);

    action_api!(
        get_group_info,
        GetGroupInfo,
        GroupIdContent,
        group_id: String
    );
    action_api!(get_group_list, GetGroupList, EmptyContent);
    action_api!(
        get_group_member_info,
        GetGroupMemberInfo,
        IdsContent,
        group_id: String,
        user_id: String
    );
    action_api!(
        get_group_member_list,
        GetGroupMemberList,
        GroupIdContent,
        group_id: String
    );
    action_api!(
        set_group_name,
        SetGroupName,
        SetGroupNameContent,
        group_id: String,
        group_name: String
    );
    action_api!(leave_group, LeaveGroup, GroupIdContent, group_id: String);
    action_api!(
        kick_group_member,
        KickGroupMember,
        IdsContent,
        group_id: String,
        user_id: String
    );
    action_api!(
        ban_group_member,
        BanGroupMember,
        IdsContent,
        group_id: String,
        user_id: String
    );
    action_api!(
        unban_group_member,
        UnbanGroupMember,
        IdsContent,
        group_id: String,
        user_id: String
    );
    action_api!(
        set_group_admin,
        SetGroupAdmin,
        IdsContent,
        group_id: String,
        user_id: String
    );
    action_api!(
        unset_group_admin,
        UnsetGroupAdmin,
        IdsContent,
        group_id: String,
        user_id: String
    );

    action_api!(
        upload_file,
        UploadFile,
        UploadFileContent,
        r#type: String,
        name: String,
        url: Option<String>,
        headers: Option<std::collections::HashMap<String, String>>,
        path: Option<String>,
        data: Option<Vec<u8>>,
        sha256: Option<String>
    );
    action_api!(
        get_file,
        GetFile,
        GetFileContent,
        file_id: String,
        r#type: String
    );
    // todo fragmented file
}

macro_rules! ext_action_api {
    ($fn_name: ident,$action_ty:ident,$content:ident) => {
        pub async fn $fn_name(&self) -> Option<ActionResp<R>> {
            self.call_action_resp(ExtendedAction::Standard(Action::$action_ty($content::default()))).await
        }
    };
    ($fn_name: ident,$action_ty:ident,$content:ident, $field_name: ident: $field_ty: ty) => {
        pub async fn $fn_name(&self, $field_name: $field_ty) -> Option<ActionResp<R>> {
            self.call_action_resp(ExtendedAction::Standard(Action::$action_ty($content{
                $field_name,
            }))).await
        }
    };
    ($fn_name: ident,$action_ty:ident,$content:ident, $($field_name: ident: $field_ty: ty),*) => {
        pub async fn $fn_name(&self, $($field_name: $field_ty,)*) -> Option<ActionResp<R>> {
            self.call_action_resp(ExtendedAction::Standard(Action::$action_ty($content{
                $($field_name,)*
            }))).await
        }
    };
}

impl<E, A, R> super::CustomOneBot<E, ExtendedAction<A>, R>
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    ext_action_api!(
        get_latest_events,
        GetLatestEvents,
        GetLatestEventsContent,
        limit: i64,
        timeout: i64
    );
    ext_action_api!(get_support_actions, GetSupportedActions, EmptyContent);
    ext_action_api!(get_status, GetStatus, EmptyContent);
    ext_action_api!(get_version, GetVersion, EmptyContent);

    ext_action_api!(
        send_message,
        SendMessage,
        SendMessageContent,
        detail_type: String,
        group_id: Option<String>,
        user_id: Option<String>,
        message: crate::Message
    );
    ext_action_api!(
        deletemessage,
        DeleteMessage,
        DeleteMessageContent,
        message_id: String
    );

    ext_action_api!(get_self_info, GetSelfInfo, EmptyContent);
    ext_action_api!(get_user_info, GetUserInfo, UserIdContent, user_id: String);
    ext_action_api!(get_friend_list, GetFriendList, EmptyContent);

    ext_action_api!(
        get_group_info,
        GetGroupInfo,
        GroupIdContent,
        group_id: String
    );
    ext_action_api!(get_group_list, GetGroupList, EmptyContent);
    ext_action_api!(
        get_group_member_info,
        GetGroupMemberInfo,
        IdsContent,
        group_id: String,
        user_id: String
    );
    ext_action_api!(
        get_group_member_list,
        GetGroupMemberList,
        GroupIdContent,
        group_id: String
    );
    ext_action_api!(
        set_group_name,
        SetGroupName,
        SetGroupNameContent,
        group_id: String,
        group_name: String
    );
    ext_action_api!(leave_group, LeaveGroup, GroupIdContent, group_id: String);
    ext_action_api!(
        kick_group_member,
        KickGroupMember,
        IdsContent,
        group_id: String,
        user_id: String
    );
    ext_action_api!(
        ban_group_member,
        BanGroupMember,
        IdsContent,
        group_id: String,
        user_id: String
    );
    ext_action_api!(
        unban_group_member,
        UnbanGroupMember,
        IdsContent,
        group_id: String,
        user_id: String
    );
    ext_action_api!(
        set_grop_admin,
        SetGroupAdmin,
        IdsContent,
        group_id: String,
        user_id: String
    );
    ext_action_api!(
        unset_grop_admin,
        UnsetGroupAdmin,
        IdsContent,
        group_id: String,
        user_id: String
    );

    ext_action_api!(
        upload_file,
        UploadFile,
        UploadFileContent,
        r#type: String,
        name: String,
        url: Option<String>,
        headers: Option<std::collections::HashMap<String, String>>,
        path: Option<String>,
        data: Option<Vec<u8>>,
        sha256: Option<String>
    );
    ext_action_api!(
        get_file,
        GetFile,
        GetFileContent,
        file_id: String,
        r#type: String
    );
    // todo fragmented file
}
