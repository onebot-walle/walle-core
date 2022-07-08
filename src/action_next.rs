use serde::{Deserialize, Serialize};

use crate::{
    prelude::WalleError,
    util::{ExtendedMap, ExtendedMapExt, ExtendedValue, PushToExtendedMap, SelfId},
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub action: String,
    pub params: ExtendedMap,
}

impl SelfId for Action {
    fn self_id(&self) -> String {
        self.params.get_downcast("self_id").unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseAction<T> {
    pub action: T,
    pub extra: ExtendedMap,
}

pub trait ActionDeclare {
    fn action() -> &'static str;
}

impl<T> From<BaseAction<T>> for Action
where
    T: ActionDeclare + PushToExtendedMap,
{
    fn from(mut action: BaseAction<T>) -> Self {
        Self {
            action: T::action().to_string(),
            params: {
                action.action.push(&mut action.extra);
                action.extra
            },
        }
    }
}

impl<T> TryFrom<Action> for BaseAction<T>
where
    T: for<'a> TryFrom<&'a mut Action, Error = WalleError> + ActionDeclare,
{
    type Error = WalleError;
    fn try_from(mut value: Action) -> Result<Self, Self::Error> {
        Ok(Self {
            action: T::try_from(&mut value)?,
            extra: value.params,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GetLatestEvents {
    pub limit: i64,
    pub timeout: i64,
}

impl ActionDeclare for GetLatestEvents {
    fn action() -> &'static str {
        "get_latest_events"
    }
}

impl PushToExtendedMap for GetLatestEvents {
    fn push(self, map: &mut ExtendedMap) {
        map.insert("limit".to_owned(), self.limit.into());
        map.insert("timeout".to_owned(), self.timeout.into());
    }
}

impl Into<ExtendedValue> for GetLatestEvents {
    fn into(self) -> ExtendedValue {
        let mut map = ExtendedMap::default();
        self.push(&mut map);
        ExtendedValue::Map(map)
    }
}

impl TryFrom<&mut Action> for GetLatestEvents {
    type Error = WalleError;
    fn try_from(action: &mut Action) -> Result<Self, Self::Error> {
        if action.action.as_str() != Self::action() {
            return Err(WalleError::DeclareNotMatch(
                Self::action(),
                action.action.clone(),
            ));
        } else {
            Ok(Self {
                limit: action.params.remove_downcast("limit")?,
                timeout: action.params.remove_downcast("timeout")?,
            })
        }
    }
}

use walle_macro::{_OneBot as OneBot, _PushToMap as PushToMap};

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[action]
pub struct DeleteMessage {
    pub message_id: Option<String>,
}

macro_rules! action {
    ($name: ident, $($f: ident: $fty: ty),*) => {
        #[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
        #[action]
        pub struct $name {
            $(pub $f: $fty,)*
        }
    };
}

use crate::util::OneBotBytes;

action!(GetUserInfo, user_id: String);
action!(GetGroupInfo, group_id: String);
action!(GetGroupMemberList, group_id: String);
action!(LeaveGroup, group_id: String);
action!(GetGroupMemberInfo, group_id: String, user_id: String);
action!(SetGroupName, group_id: String, group_name: String);
action!(GetChannelInfo, guild_id: String, channel_id: String);
action!(GetChannelList, guild_id: String);
action!(GetGuildMemberInfo, guild_id: String, user_id: String);
action!(GetGuildMemberList, guild_id: String);
action!(SetGuildName, guild_id: String, guild_name: String);
action!(
    SetChannelName,
    guild_id: String,
    channel_id: String,
    channel_name: String
);
action!(LeaveGuild, guild_id: String);
action!(GetFile, file_id: String, ty: String);
action!(GetGuildInfo, guild_id: String);

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToMap)]
#[action]
pub struct UploadFile {
    pub ty: String,
    pub name: String,
    pub url: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<OneBotBytes>,
    pub sha256: Option<String>,
}

#[test]
fn action() {
    use crate::{extended_map, WalleResult};
    let action = Action {
        action: "upload_file".to_string(),
        params: extended_map! {
            "type": "type",
            "name": "name",
            "extra": "test"
        },
    };
    let uf: WalleResult<BaseAction<UploadFile>> = action.try_into();
    println!("{:?}", uf)
}
