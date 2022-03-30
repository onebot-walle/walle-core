use crate::{action::*, ExtendedMap, ProtocolItem, WalleError, WalleResult};
use std::fmt::Debug;
use std::time::Duration;

macro_rules! action_api {
    ($fn_name: ident,$action_ty:ident,$content:ident) => {
        pub async fn $fn_name(&self) -> WalleResult<R> {
            self.call_action_resp(StandardAction::$action_ty($content::default()).into()).await
        }
    };
    ($fn_name: ident,$action_ty:ident,$content:ident, $field_name: ident: $field_ty: ty) => {
        pub async fn $fn_name(&self, $field_name: $field_ty, extra: ExtendedMap) -> WalleResult<R> {
            self.call_action_resp(StandardAction::$action_ty($content{
                $field_name, extra
            }).into()).await
        }
    };
    ($fn_name: ident,$action_ty:ident,$content:ident, $($field_name: ident: $field_ty: ty),*) => {
        pub async fn $fn_name(&self, $($field_name: $field_ty,)* extra: ExtendedMap) -> WalleResult<R> {
            self.call_action_resp(StandardAction::$action_ty($content{
                $($field_name,)* extra
            }).into()).await
        }
    };
}

impl<A, R> super::Bot<A, R> {
    pub fn new(self_id: String, sender: super::CustomActionSender<A, R>) -> Self {
        Self { self_id, sender }
    }
}

impl<A, R> super::Bot<A, R>
where
    A: ProtocolItem + From<StandardAction> + Clone + Send + 'static + Debug,
    R: ProtocolItem + Clone + Send + 'static + Debug,
{
    pub async fn call_action_resp(&self, action: A) -> WalleResult<R> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send((action, tx))
            .map_err(|_| WalleError::ActionSendError)?;
        tokio::time::timeout(Duration::from_secs(10), rx)
            .await
            .map_err(|_| WalleError::ActionResponseTimeout)?
            .map_err(WalleError::ActionResponseRecvError)
    }

    action_api!(
        get_latest_events,
        GetLatestEvents,
        GetLatestEvents,
        limit: i64,
        timeout: i64
    );
    action_api!(get_supported_actions, GetSupportedActions, ExtendedMap);
    action_api!(get_status, GetStatus, ExtendedMap);
    action_api!(get_version, GetVersion, ExtendedMap);

    action_api!(
        send_message,
        SendMessage,
        SendMessage,
        detail_type: String,
        group_id: Option<String>,
        user_id: Option<String>,
        message: crate::Message
    );
    action_api!(
        deletemessage,
        DeleteMessage,
        DeleteMessage,
        message_id: String
    );

    action_api!(get_self_info, GetSelfInfo, ExtendedMap);
    action_api!(get_user_info, GetUserInfo, GetUserInfo, user_id: String);
    action_api!(get_friend_list, GetFriendList, ExtendedMap);

    action_api!(get_group_info, GetGroupInfo, GetGroupInfo, group_id: String);
    action_api!(get_group_list, GetGroupList, ExtendedMap);
    action_api!(
        get_group_member_info,
        GetGroupMemberInfo,
        GetGroupMemberInfo,
        group_id: String,
        user_id: String
    );
    action_api!(
        get_group_member_list,
        GetGroupMemberList,
        GetGroupMemberList,
        group_id: String
    );
    action_api!(
        set_group_name,
        SetGroupName,
        SetGroupName,
        group_id: String,
        group_name: String
    );
    action_api!(leave_group, LeaveGroup, LeaveGroup, group_id: String);
    action_api!(
        kick_group_member,
        KickGroupMember,
        KickGroupMember,
        group_id: String,
        user_id: String
    );
    action_api!(
        ban_group_member,
        BanGroupMember,
        BanGroupMember,
        group_id: String,
        user_id: String
    );
    action_api!(
        unban_group_member,
        UnbanGroupMember,
        UnbanGroupMember,
        group_id: String,
        user_id: String
    );
    action_api!(
        set_grop_admin,
        SetGroupAdmin,
        SetGroupAdmin,
        group_id: String,
        user_id: String
    );
    action_api!(
        unset_grop_admin,
        UnsetGroupAdmin,
        UnsetGroupAdmin,
        group_id: String,
        user_id: String
    );

    action_api!(
        upload_file,
        UploadFile,
        UploadFile,
        r#type: String,
        name: String,
        url: Option<String>,
        headers: Option<std::collections::HashMap<String, String>>,
        path: Option<String>,
        data: Option<Vec<u8>>,
        sha256: Option<String>
    );
    action_api!(get_file, GetFile, GetFile, file_id: String, r#type: String);
    // todo fragmented file
}
