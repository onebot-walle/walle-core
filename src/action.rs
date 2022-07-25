use serde::{Deserialize, Serialize};

use crate::{
    prelude::WalleError,
    util::{PushToValueMap, SelfId, Value, ValueMap, ValueMapExt},
    value_map,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub action: String,
    pub params: ValueMap,
}

impl ValueMapExt for Action {
    fn get_downcast<T>(&self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.params.get_downcast(key)
    }
    fn remove_downcast<T>(&mut self, key: &str) -> Result<T, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.params.remove_downcast(key)
    }
    fn try_get_downcast<T>(&self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.params.try_get_downcast(key)
    }
    fn try_remove_downcast<T>(&mut self, key: &str) -> Result<Option<T>, WalleError>
    where
        T: TryFrom<Value, Error = WalleError>,
    {
        self.params.try_remove_downcast(key)
    }
    fn push<T>(&mut self, value: T)
    where
        T: PushToValueMap,
    {
        value.push_to(&mut self.params)
    }
}

impl SelfId for Action {
    fn self_id(&self) -> String {
        self.params.get_downcast("self_id").unwrap_or_default()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct BaseAction<T> {
    pub action: T,
    pub extra: ValueMap,
}

pub trait ActionDeclare {
    fn action(&self) -> &'static str;
    fn check(action: &Action) -> bool;
}

impl<T> From<BaseAction<T>> for Action
where
    T: ActionDeclare + PushToValueMap,
{
    fn from(mut action: BaseAction<T>) -> Self {
        Self {
            action: action.action.action().to_string(),
            params: {
                action.action.push_to(&mut action.extra);
                action.extra
            },
        }
    }
}

impl<T> TryFrom<Action> for BaseAction<T>
where
    T: for<'a> TryFrom<&'a mut Action, Error = WalleError>,
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
    fn action(&self) -> &'static str {
        "get_latest_events"
    }
    fn check(action: &Action) -> bool {
        action.action.as_str() == "get_latest_events"
    }
}

impl PushToValueMap for GetLatestEvents {
    fn push_to(self, map: &mut ValueMap) {
        map.insert("limit".to_owned(), self.limit.into());
        map.insert("timeout".to_owned(), self.timeout.into());
    }
}

impl Into<Value> for GetLatestEvents {
    fn into(self) -> Value {
        let mut map = ValueMap::default();
        self.push_to(&mut map);
        Value::Map(map)
    }
}

impl TryFrom<&mut Action> for GetLatestEvents {
    type Error = WalleError;
    fn try_from(action: &mut Action) -> Result<Self, Self::Error> {
        if action.action.as_str() != "get_latest_events" {
            return Err(WalleError::DeclareNotMatch(
                "get_latest_events",
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

use walle_macro::{_OneBot as OneBot, _PushToValueMap as PushToValueMap};

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[action]
pub struct DeleteMessage {
    pub message_id: String,
}

macro_rules! action {
    ($name: ident, $($f: ident: $fty: ty),*) => {
        #[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
        #[action]
        pub struct $name {
            $(pub $f: $fty,)*
        }
    };
}

use crate::util::OneBotBytes;

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[action]
pub struct GetUserInfo {
    pub user_id: String,
}

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
action!(GetGuildInfo, guild_id: String);

#[derive(Debug, Clone, PartialEq, OneBot, PushToValueMap)]
#[action]
pub struct SendMessage {
    pub detail_type: String,
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub message: crate::segment::Segments,
}

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[action]
#[value]
pub struct GetFile {
    pub file_id: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq, OneBot, PushToValueMap)]
#[action]
#[value]
pub struct UploadFile {
    pub ty: String,
    pub name: String,
    pub url: Option<String>,
    pub headers: Option<std::collections::HashMap<String, String>>,
    pub path: Option<String>,
    pub data: Option<OneBotBytes>,
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UploadFileFragmented {
    Prepare {
        name: String,
        total_size: i64,
    },
    Transfer {
        file_id: String,
        offset: i64,
        size: i64,
        data: OneBotBytes,
    },
    Finish {
        file_id: String,
        sha256: Option<String>,
    },
}

impl ActionDeclare for UploadFileFragmented {
    fn action(&self) -> &'static str {
        "upload_file_fragmented"
    }
    fn check(action: &Action) -> bool {
        action.action.as_str() == "upload_file_fragmented"
    }
}

impl TryFrom<&mut Action> for UploadFileFragmented {
    type Error = WalleError;
    fn try_from(action: &mut Action) -> Result<Self, Self::Error> {
        if Self::check(&action) {
            Err(WalleError::DeclareNotMatch(
                "upload_file_fragmented",
                action.action.clone(),
            ))
        } else {
            match action.params.remove_downcast::<String>("stage")?.as_str() {
                "prepare" => Ok(Self::Prepare {
                    name: action.params.remove_downcast("name")?,
                    total_size: action.params.remove_downcast("total_size")?,
                }),
                "transfer" => Ok(Self::Transfer {
                    file_id: action.params.remove_downcast("file_id")?,
                    offset: action.params.remove_downcast("offset")?,
                    size: action.params.remove_downcast("size")?,
                    data: action.params.remove_downcast("data")?,
                }),
                "finish" => Ok(Self::Finish {
                    file_id: action.params.remove_downcast("file_id")?,
                    sha256: action.params.try_remove_downcast("sha256")?,
                }),
                x => Err(WalleError::DeclareNotMatch(
                    "prepare or transfer or finish",
                    x.to_string(),
                )),
            }
        }
    }
}

impl TryFrom<Action> for UploadFileFragmented {
    type Error = WalleError;
    fn try_from(mut value: Action) -> Result<Self, Self::Error> {
        Self::try_from(&mut value)
    }
}

impl From<UploadFileFragmented> for Action {
    fn from(u: UploadFileFragmented) -> Self {
        Self {
            action: u.action().to_string(),
            params: {
                match u {
                    UploadFileFragmented::Prepare { name, total_size } => value_map! {
                        "stage": "prepare",
                        "name": name,
                        "total_size": total_size
                    },
                    UploadFileFragmented::Transfer {
                        file_id,
                        offset,
                        size,
                        data,
                    } => value_map! {
                        "stage" : "transfer",
                        "file_id" : file_id,
                        "offset" : offset,
                        "size" : size,
                        "date" : data
                    },
                    UploadFileFragmented::Finish { file_id, sha256 } => value_map! {
                        "stage" : "finish",
                        "file_id" : file_id,
                        "sha256" : sha256
                    },
                }
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

impl ActionDeclare for GetFileFragmented {
    fn action(&self) -> &'static str {
        "get_file_fragmented"
    }
    fn check(action: &Action) -> bool {
        action.action.as_str() == "get_file_fragmented"
    }
}

impl TryFrom<&mut Action> for GetFileFragmented {
    type Error = WalleError;
    fn try_from(action: &mut Action) -> Result<Self, Self::Error> {
        if Self::check(action) {
            Err(WalleError::DeclareNotMatch(
                "get_file_fragmented",
                action.action.clone(),
            ))
        } else {
            match action.params.remove_downcast::<String>("stage")?.as_str() {
                "prepare" => Ok(Self::Prepare {
                    file_id: action.params.remove_downcast("file_id")?,
                }),
                "transfer" => Ok(Self::Transfer {
                    file_id: action.params.remove_downcast("file_id")?,
                    offset: action.params.remove_downcast("offset")?,
                    size: action.params.remove_downcast("size")?,
                }),
                x => Err(WalleError::DeclareNotMatch(
                    "prepare or transfer or finish",
                    x.to_string(),
                )),
            }
        }
    }
}

impl TryFrom<Action> for GetFileFragmented {
    type Error = WalleError;
    fn try_from(mut value: Action) -> Result<Self, Self::Error> {
        Self::try_from(&mut value)
    }
}

impl From<GetFileFragmented> for Action {
    fn from(g: GetFileFragmented) -> Self {
        Self {
            action: g.action().to_string(),
            params: match g {
                GetFileFragmented::Prepare { file_id } => value_map! {
                    "stage": "prepare",
                    "file_id": file_id
                },
                GetFileFragmented::Transfer {
                    file_id,
                    offset,
                    size,
                } => value_map! {
                    "stage": "transfer",
                    "file_id": file_id,
                    "offset": offset,
                    "size": size
                },
            },
        }
    }
}

#[test]
fn action() {
    use crate::{value_map, WalleResult};
    let action = Action {
        action: "upload_file".to_string(),
        params: value_map! {
            "type": "type",
            "name": "name",
            "extra": "test"
        },
    };
    let uf: WalleResult<BaseAction<UploadFile>> = action.try_into();
    println!("{:?}", uf);
}
