//! Action 相关模型定义
use serde::{Deserialize, Serialize};

use crate::{
    prelude::{WalleError, WalleResult},
    structs::Selft,
    util::{GetSelf, PushToValueMap, ValueMap, ValueMapExt},
};

/// 标准 Action 模型
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Action {
    pub action: String,
    pub params: ValueMap,
    #[serde(rename = "self")]
    pub selft: Option<Selft>,
}

pub trait ToAction: PushToValueMap {
    fn ty(&self) -> &'static str;
    fn selft(&self) -> Option<Selft> {
        None
    }
    fn to_action(self) -> Action
    where
        Self: Sized,
    {
        Action {
            action: self.ty().to_string(),
            selft: self.selft(),
            params: {
                let mut map = ValueMap::new();
                self.push_to(&mut map);
                map
            },
        }
    }
}

pub trait TryFromAction: Sized {
    fn try_from_action_mut(action: &mut Action) -> WalleResult<Self>;
    fn try_from_action(mut action: Action) -> WalleResult<Self> {
        Self::try_from_action_mut(&mut action)
    }
}

impl GetSelf for Action {
    fn get_self(&self) -> Selft {
        self.selft.clone().unwrap_or_default()
    }
}

/// 泛型可扩展 Action 模型
#[derive(Debug, Clone, PartialEq)]
pub struct BaseAction<T> {
    pub action: T,
    pub selft: Option<Selft>,
    pub extra: ValueMap,
}

impl<T> From<BaseAction<T>> for Action
where
    T: ToAction,
{
    fn from(mut action: BaseAction<T>) -> Self {
        Self {
            action: action.action.ty().to_string(),
            selft: action.selft,
            params: {
                action.action.push_to(&mut action.extra);
                action.extra
            },
        }
    }
}

impl<T> From<(T, Selft)> for Action
where
    T: ToAction + Into<ValueMap>,
{
    fn from(v: (T, Selft)) -> Self {
        Self {
            action: v.0.ty().to_owned(),
            params: v.0.into(),
            selft: Some(v.1),
        }
    }
}

impl<T> TryFrom<Action> for BaseAction<T>
where
    T: TryFromAction,
{
    type Error = WalleError;
    fn try_from(mut value: Action) -> Result<Self, Self::Error> {
        Ok(Self {
            action: T::try_from_action_mut(&mut value)?,
            selft: value.selft,
            extra: value.params,
        })
    }
}

use walle_macro::{
    _PushToValueMap as PushToValueMap, _ToAction as ToAction, _TryFromAction as TryFromAction,
    _TryFromValue as TryFromValue,
};

#[derive(Debug, Clone, PartialEq, Eq, TryFromValue, TryFromAction, ToAction, PushToValueMap)]
pub struct GetLatestEvents {
    pub limit: i64,
    pub timeout: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, TryFromValue, TryFromAction, ToAction, PushToValueMap)]
pub struct DeleteMessage {
    pub message_id: String,
}

macro_rules! action {
    ($name: ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, TryFromValue, TryFromAction, ToAction, PushToValueMap)]
        pub struct $name;
    };
    ($name: ident, $($f: ident: $fty: ty),*) => {
        #[derive(Debug, Clone, PartialEq, Eq, TryFromValue, TryFromAction, ToAction, PushToValueMap)]
        pub struct $name {
            $(pub $f: $fty,)*
        }
    };
}

use crate::util::OneBotBytes;

#[derive(Debug, Clone, PartialEq, Eq, TryFromValue, TryFromAction, ToAction, PushToValueMap)]
pub struct GetUserInfo {
    pub user_id: String,
}
// Group
action!(GetGroupInfo, group_id: String);
action!(GetGroupList);
action!(GetGroupMemberInfo, group_id: String, user_id: String);
action!(GetGroupMemberList, group_id: String);
action!(SetGroupName, group_id: String, group_name: String);
action!(LeaveGroup, group_id: String);
// Guild
action!(GetGuildInfo, guild_id: String);
action!(GetGuildList);
action!(SetGuildName, guild_id: String, guild_name: String);
action!(GetGuildMemberInfo, guild_id: String, user_id: String);
action!(GetGuildMemberList, guild_id: String);
action!(LeaveGuild, guild_id: String);
// Channel
action!(GetChannelInfo, guild_id: String, channel_id: String);
action!(GetChannelList, guild_id: String, joined_only: bool);
action!(
    SetChannelName,
    guild_id: String,
    channel_id: String,
    channel_name: String
);
action!(
    GetChannelMemberInfo,
    guild_id: String,
    channel_id: String,
    user_id: String
);
action!(GetChannelMemberList, guild_id: String, channel_id: String);
action!(LeaveChannel, guild_id: String, channel_id: String);
// message
#[derive(Debug, Clone, PartialEq, TryFromValue, TryFromAction, ToAction, PushToValueMap)]
pub struct SendMessage {
    pub detail_type: String,
    pub user_id: Option<String>,
    pub group_id: Option<String>,
    pub guild_id: Option<String>,
    pub channel_id: Option<String>,
    pub message: crate::segment::Segments,
}

#[derive(Debug, Clone, PartialEq, Eq, TryFromValue, TryFromAction, ToAction, PushToValueMap)]
pub struct GetFile {
    pub file_id: String,
    pub ty: String,
}

#[derive(Debug, Clone, PartialEq, Eq, TryFromValue, TryFromAction, ToAction, PushToValueMap)]
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
        data: OneBotBytes,
    },
    Finish {
        file_id: String,
        sha256: Option<String>,
    },
}

impl TryFromAction for UploadFileFragmented {
    fn try_from_action_mut(action: &mut Action) -> WalleResult<Self> {
        if action.action != "upload_file_fragmented" {
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

impl TryFrom<&mut ValueMap> for UploadFileFragmented {
    type Error = WalleError;
    fn try_from(map: &mut ValueMap) -> Result<Self, Self::Error> {
        match map.remove_downcast::<String>("stage")?.as_str() {
            "prepare" => Ok(Self::Prepare {
                name: map.remove_downcast("name")?,
                total_size: map.remove_downcast("total_size")?,
            }),
            "transfer" => Ok(Self::Transfer {
                file_id: map.remove_downcast("file_id")?,
                offset: map.remove_downcast("offset")?,
                data: map.remove_downcast("data")?,
            }),
            "finish" => Ok(Self::Finish {
                file_id: map.remove_downcast("file_id")?,
                sha256: map.try_remove_downcast("sha256")?,
            }),
            x => Err(WalleError::DeclareNotMatch(
                "prepare or transfer or finish",
                x.to_string(),
            )),
        }
    }
}

impl PushToValueMap for UploadFileFragmented {
    fn push_to(self, map: &mut ValueMap) {
        match self {
            UploadFileFragmented::Prepare { name, total_size } => {
                map.insert("stage".to_string(), "prepare".into());
                map.insert("name".to_string(), name.into());
                map.insert("total_size".to_string(), total_size.into());
            }
            UploadFileFragmented::Transfer {
                file_id,
                offset,
                data,
            } => {
                map.insert("stage".to_string(), "transfer".into());
                map.insert("file_id".to_string(), file_id.into());
                map.insert("offset".to_string(), offset.into());
                map.insert("data".to_string(), data.into());
            }
            UploadFileFragmented::Finish { file_id, sha256 } => {
                map.insert("stage".to_string(), "finish".into());
                map.insert("file_id".to_string(), file_id.into());
                map.insert("sha256".to_string(), sha256.into());
            }
        }
    }
}

impl ToAction for UploadFileFragmented {
    fn ty(&self) -> &'static str {
        "upload_file_fragmented"
    }
}

impl From<UploadFileFragmented> for Action {
    fn from(u: UploadFileFragmented) -> Self {
        u.to_action()
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

impl TryFromAction for GetFileFragmented {
    fn try_from_action_mut(action: &mut Action) -> WalleResult<Self> {
        if action.action != "get_file_fragmented" {
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

impl TryFrom<&mut ValueMap> for GetFileFragmented {
    type Error = WalleError;
    fn try_from(map: &mut ValueMap) -> WalleResult<Self> {
        match map.remove_downcast::<String>("stage")?.as_str() {
            "prepare" => Ok(Self::Prepare {
                file_id: map.remove_downcast("file_id")?,
            }),
            "transfer" => Ok(Self::Transfer {
                file_id: map.remove_downcast("file_id")?,
                offset: map.remove_downcast("offset")?,
                size: map.remove_downcast("size")?,
            }),
            x => Err(WalleError::DeclareNotMatch(
                "prepare | transfer | finish",
                x.to_string(),
            )),
        }
    }
}

impl PushToValueMap for GetFileFragmented {
    fn push_to(self, map: &mut ValueMap) {
        match self {
            Self::Prepare { file_id } => {
                map.insert("stage".to_string(), "prepare".into());
                map.insert("file_id".to_string(), file_id.into());
            }
            Self::Transfer {
                file_id,
                offset,
                size,
            } => {
                map.insert("stage".to_string(), "transfer".into());
                map.insert("file_id".to_string(), file_id.into());
                map.insert("offset".to_string(), offset.into());
                map.insert("size".to_string(), size.into());
            }
        }
    }
}

impl ToAction for GetFileFragmented {
    fn ty(&self) -> &'static str {
        "get_file_fragmented"
    }
}

impl From<GetFileFragmented> for Action {
    fn from(g: GetFileFragmented) -> Self {
        g.to_action()
    }
}

#[test]
fn action() {
    use crate::{value_map, WalleResult};
    let action = Action {
        action: "upload_file".to_string(),
        selft: None,
        params: value_map! {
            "type": "type",
            "name": "name",
            "extra": "test"
        },
    };
    let uf: WalleResult<BaseAction<UploadFile>> = action.try_into();
    println!("{:?}", uf);
}
