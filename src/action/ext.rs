use std::{future::Future, pin::Pin};

use super::UploadFile;
use crate::resp::*;
use crate::Message;
use crate::WalleResult;
use crate::{ExtendedMap, ExtendedValue};

type Pbfr<'r, R> = Pin<Box<dyn Future<Output = WalleResult<R>> + Send + 'r>>;

macro_rules! exts {
    ($ex_name: ident, $name: ident, $rty: ty) => {
        fn $ex_name<'a, 'b>(
            &'a self,
            extra: ExtendedMap,
        ) -> Pbfr<'b, $rty>
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
        ) -> Pbfr<'b, $rty>
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
        ) -> Pbfr<'b, $rty>
        where
            'a: 'b,
            Self: 'b,
            $rty: TryFrom<R>;
        exts_noex!($ex_name, $name, $rty, $($field_name: $field_ty),*);
    };
}

macro_rules! exts_noex {
    ($ex_name: ident, $name: ident, $rty: ty) => {
        fn $name<'a, 'b>(&'a self) -> Pbfr<'b, $rty>
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
        ) -> Pbfr<'b, $rty>
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
        ) -> Pbfr<'b, $rty>
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
        guild_id: Option<String>,
        channel_id: Option<String>,
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
        get_guild_info_ex,
        get_guild_info,
        Resp<GuildInfoContent>,
        guild_id: String
    );
    exts!(
        get_guild_list_ex,
        get_guild_list,
        Resp<Vec<GuildInfoContent>>
    );
    exts!(
        get_channel_info_ex,
        get_channel_info,
        Resp<ChannelInfoContent>,
        guild_id: String,
        channel_id: String
    );
    exts!(
        get_channel_list_ex,
        get_channel_list,
        Resp<Vec<ChannelInfoContent>>,
        guild_id: String
    );
    exts!(
        get_guild_member_info_ex,
        get_guild_member_info,
        Resp<UserInfoContent>,
        guild_id: String,
        user_id: String
    );
    exts!(
        get_guild_member_list_ex,
        get_guild_member_list,
        Resp<Vec<UserInfoContent>>,
        guild_id: String
    );
    exts!(
        set_guild_name_ex,
        set_guild_name,
        Resp<ExtendedValue>,
        guild_id: String,
        guild_name: String
    );
    exts!(
        set_channel_name_ex,
        set_channel_name,
        Resp<ExtendedValue>,
        guild_id: String,
        channel_id: String,
        channel_name: String
    );
    exts!(
        leave_guild_ex,
        leave_guild,
        Resp<ExtendedValue>,
        guild_id: String
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
    ) -> Pbfr<'b, Resp<SendMessageRespContent>>
    where
        'a: 'b,
        Self: 'b,
        Resp<SendMessageRespContent>: TryFrom<R>,
    {
        self.send_message_ex(
            "private".to_string(),
            None,
            Some(user_id),
            None,
            None,
            message,
            extra,
        )
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
    ) -> Pbfr<'b, Resp<SendMessageRespContent>>
    where
        'a: 'b,
        Self: 'b,
        Resp<SendMessageRespContent>: TryFrom<R>,
    {
        self.send_message_ex(
            "group".to_string(),
            Some(group_id),
            None,
            None,
            None,
            message,
            extra,
        )
    }
    exts_noex!(
        send_group_msg_ex,
        send_group_msg,
        Resp<SendMessageRespContent>,
        group_id: String,
        message: Message
    );
    fn send_channel_msg_ex<'a, 'b>(
        &'a self,
        guild_id: String,
        channel_id: String,
        message: Message,
        extra: ExtendedMap,
    ) -> Pbfr<'b, Resp<SendMessageRespContent>>
    where
        'a: 'b,
        Self: 'b,
        Resp<SendMessageRespContent>: TryFrom<R>,
    {
        self.send_message_ex(
            "channel".to_string(),
            None,
            None,
            Some(guild_id),
            Some(channel_id),
            message,
            extra,
        )
    }
    exts_noex!(
        send_channel_msg_ex,
        send_channel_msg,
        Resp<SendMessageRespContent>,
        guild_id: String,
        channel_id: String,
        message: Message
    );
    fn upload_file_by_url_ex<'a, 'b>(
        &'a self,
        name: String,
        url: String,
        headers: std::collections::HashMap<String, String>,
        sha256: Option<String>,
        extra: ExtendedMap,
    ) -> Pbfr<'b, Resp<FileIdContent>>
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
    ) -> Pbfr<'b, Resp<FileIdContent>>
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
    ) -> Pbfr<'b, Resp<FileIdContent>>
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
