#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

/// Onebot Action
pub mod action;
/// Onebot ActionResp
pub mod action_resp;
mod comms;
/// 相关配置项
pub mod config;
/// Onebot Event
pub mod event;
mod event_builder;
mod handle;
#[cfg(feature = "impl")]
#[cfg_attr(docsrs, doc(cfg(feature = "impl")))]
/// 实现端相关 api
pub mod impls;
mod message;
#[cfg(feature = "sdk")]
#[cfg_attr(docsrs, doc(cfg(feature = "sdk")))]
/// 应用端相关 api
pub mod sdk;
mod test;
pub(crate) mod utils;

pub use action::Action;
pub use action_resp::{ActionResp, ActionRespContent, ActionResps};
pub use config::*;
pub use event::{
    Event, EventContent, MessageEvent, MessageEventType, MetaEvent, NoticeEvent, RequestEvent,
};
pub use handle::{ActionHandler, ActionRespHandler, DefaultHandler, EventHandler};
pub use message::{Message, MessageBuild, MessageSegment};

pub use async_trait::async_trait;

use serde::{Deserialize, Serialize};
/// 空结构体，用于对应 Json 中的空 Map
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct EmptyContent {}
