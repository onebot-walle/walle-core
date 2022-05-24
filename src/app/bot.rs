use crate::action::BotActionExt;
use crate::resp::*;
use crate::{action::*, ExtendedMap, ExtendedValue, WalleError, WalleResult};
use crate::{Message, Resp};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

macro_rules! exts {
    ($fn_name: ident, $content:ident, $rty: ty) => {
        fn $fn_name<'a, 'b>(&'a self, extra: ExtendedMap)
            -> Pin<Box<dyn Future<Output = WalleResult<$rty>> + Send + 'b>>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>,
        {
            Box::pin(async move {
                self.call_action(StandardAction::$content(extra).into())
                    .await?
                    .to_result()
                    .map_err(WalleError::RespError)?
                    .try_into()
                    .map_err(|_| WalleError::RespMissmatch)
            })
        }
    };
    ($fn_name: ident, $content:ident, $rty: ty, $field_name: ident: $field_ty: ty) => {
        fn $fn_name<'a, 'b>(&'a self, $field_name: $field_ty, extra: ExtendedMap)
            -> Pin<Box<dyn Future<Output = WalleResult<$rty>> + Send + 'b>>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>,
        {
            Box::pin(async move {
                self.call_action(StandardAction::$content($content{
                    $field_name, extra
                }).into())
                    .await?
                    .to_result()
                    .map_err(WalleError::RespError)?
                    .try_into()
                    .map_err(|_| WalleError::RespMissmatch)
            })
        }
    };
    ($fn_name: ident, $content:ident, $rty: ty, $($field_name: ident: $field_ty: ty),*) => {
        fn $fn_name<'a, 'b>(&'a self, $($field_name: $field_ty,)* extra: ExtendedMap)
            -> Pin<Box<dyn Future<Output = WalleResult<$rty>> + Send + 'b>>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>,
        {
            Box::pin(async move {
                self.call_action(StandardAction::$content($content{
                    $($field_name,)* extra
                }).into())
                    .await?
                    .to_result()
                    .map_err(WalleError::RespError)?
                    .try_into()
                    .map_err(|_| WalleError::RespMissmatch)
            })
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

#[async_trait::async_trait]
impl<A, R> BotActionExt<R> for super::Bot<A, R>
where
    A: From<StandardAction> + Clone + Send + Sync + 'static,
    R: RespExt<Error = RespError> + Send + Sync + 'static,
{
    // exts!(
    //     get_latest_events_ex,
    //     GetLatestEvents,
    //     limit: i64,
    //     timeout: i64
    // );
    exts!(
        get_supported_actions_ex,
        GetSupportedActions,
        Resp<Vec<String>>
    );
    exts!(get_status_ex, GetStatus, Resp<StatusContent>);
    exts!(get_version_ex, GetVersion, Resp<VersionContent>);
    exts!(
        send_message_ex,
        SendMessage,
        Resp<SendMessageRespContent>,
        detail_type: String,
        group_id: Option<String>,
        user_id: Option<String>,
        guild_id: Option<String>,
        channel_id: Option<String>,
        message: Message
    );
    exts!(
        delete_message_ex,
        DeleteMessage,
        Resp<ExtendedValue>,
        message_id: String
    );
    exts!(get_self_info_ex, GetSelfInfo, Resp<UserInfoContent>);
    exts!(
        get_user_info_ex,
        GetUserInfo,
        Resp<UserInfoContent>,
        user_id: String
    );
    exts!(
        get_friend_list_ex,
        GetFriendList,
        Resp<Vec<UserInfoContent>>
    );
    exts!(
        get_group_info_ex,
        GetGroupInfo,
        Resp<GroupInfoContent>,
        group_id: String
    );
    exts!(get_group_list_ex, GetGroupList, Resp<Vec<GroupInfoContent>>);
    exts!(
        get_group_member_info_ex,
        GetGroupMemberInfo,
        Resp<UserInfoContent>,
        group_id: String,
        user_id: String
    );
    exts!(
        get_group_member_list_ex,
        GetGroupMemberList,
        Resp<Vec<GroupInfoContent>>,
        group_id: String
    );
    exts!(
        set_group_name_ex,
        SetGroupName,
        Resp<ExtendedValue>,
        group_id: String,
        group_name: String
    );
    exts!(
        leave_group_ex,
        LeaveGroup,
        Resp<ExtendedValue>,
        group_id: String
    );
    exts!(
        kick_group_member_ex,
        KickGroupMember,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        ban_group_member_ex,
        BanGroupMember,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        unban_group_member_ex,
        UnbanGroupMember,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        set_group_admin_ex,
        SetGroupAdmin,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        unset_group_admin_ex,
        UnsetGroupAdmin,
        Resp<ExtendedValue>,
        group_id: String,
        user_id: String
    );
    exts!(
        get_guild_info_ex,
        GetGuildInfo,
        Resp<GuildInfoContent>,
        guild_id: String
    );
    exts!(get_guild_list_ex, GetGuildList, Resp<Vec<GuildInfoContent>>);
    exts!(
        get_channel_info_ex,
        GetChannelInfo,
        Resp<ChannelInfoContent>,
        guild_id: String,
        channel_id: String
    );
    exts!(
        get_channel_list_ex,
        GetChannelList,
        Resp<Vec<ChannelInfoContent>>,
        guild_id: String
    );
    exts!(
        get_guild_member_info_ex,
        GetGuildMemberInfo,
        Resp<UserInfoContent>,
        guild_id: String,
        user_id: String
    );
    exts!(
        get_guild_member_list_ex,
        GetGuildMemberList,
        Resp<Vec<UserInfoContent>>,
        guild_id: String
    );
    exts!(
        set_guild_name_ex,
        SetGuildName,
        Resp<ExtendedValue>,
        guild_id: String,
        guild_name: String
    );
    exts!(
        set_channel_name_ex,
        SetChannelName,
        Resp<ExtendedValue>,
        guild_id: String,
        channel_id: String,
        channel_name: String
    );
    exts!(
        leave_guild_ex,
        LeaveGuild,
        Resp<ExtendedValue>,
        guild_id: String
    );
    exts!(
        upload_file_ex,
        UploadFile,
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
        GetFile,
        Resp<UploadFile>,
        file_id: String,
        r#type: String
    );
    async fn upload_file_fragmented(
        &self,
        name: String,
        mut file: tokio::fs::File,
    ) -> WalleResult<Resp<FileIdContent>>
    where
        Resp<FileIdContent>: TryFrom<R>,
    {
        use sha2::{Digest, Sha256};
        use tokio::io::AsyncReadExt;
        const CHUNK_SIZE: usize = 1024 * 1024;
        let meta_data = file.metadata().await?;
        let total_size = meta_data.len();
        let file_content: Resp<FileIdContent> = self
            .call_action(
                StandardAction::UploadFileFragmented(UploadFileFragmented::Prepare {
                    name,
                    total_size: total_size as i64,
                })
                .into(),
            )
            .await?
            .to_result()
            .map_err(WalleError::RespError)?
            .try_into()
            .map_err(|_| WalleError::RespMissmatch)?;
        let file_id = file_content.data.file_id;
        let mut cache = Vec::with_capacity(CHUNK_SIZE);
        let mut hasher = Sha256::new();
        let mut t = 0;
        while file.read_buf(&mut cache).await? > 0 {
            hasher.update(&cache);
            self.call_action(
                StandardAction::UploadFileFragmented(UploadFileFragmented::Transfer {
                    file_id: file_id.clone(),
                    offset: t * CHUNK_SIZE as i64,
                    size: cache.len() as i64,
                    data: cache.clone(),
                })
                .into(),
            )
            .await?
            .to_result()
            .map_err(WalleError::RespError)?;
            cache.clear();
            t += 1;
        }
        let sha256 = hex::encode(&hasher.finalize());
        self.call_action(
            StandardAction::UploadFileFragmented(UploadFileFragmented::Finish { file_id, sha256 })
                .into(),
        )
        .await?
        .to_result()
        .map_err(WalleError::RespError)?
        .try_into()
        .map_err(|_| WalleError::RespMissmatch)
    }
}
