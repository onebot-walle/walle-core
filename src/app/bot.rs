use crate::action::BotActionExt;
use crate::Message;
use crate::{action::*, ExtendedMap, WalleError, WalleResult};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

macro_rules! exts {
    ($fn_name: ident, $content:ident) => {
        fn $fn_name<'a, 'b>(&'a self, extra: ExtendedMap) -> Pin<Box<dyn Future<Output = WalleResult<R>> + Send + 'b>>
        where
            'a: 'b,
            Self: 'b,
        {
            Box::pin(self.call_action(StandardAction::$content(extra).into()))
        }
    };
    ($fn_name: ident, $content:ident, $field_name: ident: $field_ty: ty) => {
        fn $fn_name<'a, 'b>(&'a self, $field_name: $field_ty, extra: ExtendedMap)  -> Pin<Box<dyn Future<Output = WalleResult<R>> + Send + 'b>> where 'a: 'b, Self: 'b, {
            Box::pin(self.call_action(StandardAction::$content($content{
                $field_name, extra
            }).into()))
        }
    };
    ($fn_name: ident, $content:ident, $($field_name: ident: $field_ty: ty),*) => {
        fn $fn_name<'a, 'b>(&'a self, $($field_name: $field_ty,)* extra: ExtendedMap)  -> Pin<Box<dyn Future<Output = WalleResult<R>> + Send + 'b>> where 'a: 'b, Self: 'b, {
            Box::pin(self.call_action(StandardAction::$content($content{
                $($field_name,)* extra
            }).into()))
        }
    };
}

impl<A, R> super::Bot<A, R>
where
    A: Clone,
{
    pub fn new(self_id: String, sender: super::CustomActionSender<A, R>) -> Self {
        Self { self_id, sender }
    }

    pub async fn call_action(&self, action: A) -> WalleResult<R> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send((action.clone(), Some(tx)))
            .map_err(|_| WalleError::ActionSendError)?;
        Ok(tokio::time::timeout(Duration::from_secs(10), rx)
            .await
            .map_err(|_| {
                self.sender.send((action, None)).ok();
                WalleError::ResponseTimeout
            })?
            .unwrap())
    }
}

impl<A, R> BotActionExt<R> for super::Bot<A, R>
where
    A: From<StandardAction> + Clone + Send + Sync + 'static,
    R: Send + Sync + 'static,
{
    exts!(
        get_latest_events_ex,
        GetLatestEvents,
        limit: i64,
        timeout: i64
    );
    exts!(get_supported_actions_ex, GetSupportedActions);
    exts!(get_status_ex, GetStatus);
    exts!(get_version_ex, GetVersion);
    exts!(
        send_message_ex,
        SendMessage,
        detail_type: String,
        group_id: Option<String>,
        user_id: Option<String>,
        message: Message
    );
    exts!(delete_message_ex, DeleteMessage, message_id: String);
    exts!(get_message_ex, GetMessage, message_id: String);
    exts!(get_self_info_ex, GetSelfInfo);
    exts!(get_user_info_ex, GetUserInfo, user_id: String);
    exts!(get_friend_list_ex, GetFriendList);
    exts!(get_group_info_ex, GetGroupInfo, group_id: String);
    exts!(get_group_list_ex, GetGroupList);
    exts!(
        get_group_member_info_ex,
        GetGroupMemberInfo,
        group_id: String,
        user_id: String
    );
    exts!(
        get_group_member_list_ex,
        GetGroupMemberList,
        group_id: String
    );
    exts!(
        set_group_name_ex,
        SetGroupName,
        group_id: String,
        group_name: String
    );
    exts!(leave_group_ex, LeaveGroup, group_id: String);
    exts!(
        kick_group_member_ex,
        KickGroupMember,
        group_id: String,
        user_id: String
    );
    exts!(
        ban_group_member_ex,
        BanGroupMember,
        group_id: String,
        user_id: String
    );
    exts!(
        unban_group_member_ex,
        UnbanGroupMember,
        group_id: String,
        user_id: String
    );
    exts!(
        set_group_admin_ex,
        SetGroupAdmin,
        group_id: String,
        user_id: String
    );
    exts!(
        unset_group_admin_ex,
        UnsetGroupAdmin,
        group_id: String,
        user_id: String
    );
}
