#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

/// Onebot Action
pub mod action;
/// Onebot ActionResp
pub mod action_resp;
#[cfg(feature = "app")]
#[cfg_attr(docsrs, doc(cfg(feature = "app")))]
/// 应用端相关 api
pub mod app;
mod comms;
/// 相关配置项
pub mod config;
mod error;
/// Onebot Event
pub mod event;
mod event_builder;
mod handle;
#[cfg(feature = "impl")]
#[cfg_attr(docsrs, doc(cfg(feature = "impl")))]
/// 实现端相关 api
pub mod impls;
mod message;
mod test;
mod utils;

pub use action::Action;
pub use action_resp::{ActionResp, ActionRespContent, ActionResps};
pub use config::*;
pub use error::*;
pub use event::*;
#[cfg(feature = "app")]
pub use handle::EventHandler;
pub use handle::{ActionHandler, DefaultHandler};
pub use message::{Message, MessageAlt, MessageBuild, MessageSegment};
pub use utils::*;

pub use async_trait::async_trait;

use serde::{de::Visitor, Deserialize, Serialize};

/// 空结构体，用于对应 Json 中的空 Map
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmptyContent {}

/// 默认 serde 失败的 Struct 用于非扩展占位
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AlwaysFailSturct;

struct AFSVisitor;

impl<'de> Visitor<'de> for AFSVisitor {
    type Value = AlwaysFailSturct;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("Nothing wrong this is a AlwaysFailSturct")
    }
}

impl<'de> Deserialize<'de> for AlwaysFailSturct {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_any(AFSVisitor)
    }
}

impl Serialize for AlwaysFailSturct {
    fn serialize<S>(&self, _: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        Err(serde::ser::Error::custom(
            "Nothing wrong this is a AlwaysFailSturct",
        ))
    }
}

#[allow(dead_code)]
static SHUTDOWN: u8 = 0;
#[allow(dead_code)]
static RUNNING: u8 = 1;
