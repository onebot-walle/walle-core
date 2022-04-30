#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[allow(dead_code)]
const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Onebot Action
pub mod action;
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
mod handle;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
mod hooks;
#[cfg(feature = "impl")]
#[cfg_attr(docsrs, doc(cfg(feature = "impl")))]
/// 实现端相关 api
pub mod impls;
mod message;
/// Onebot ActionResp
pub mod resp;
mod test;
mod utils;

use std::env;

pub use action::StandardAction;
pub use config::*;
pub use error::*;
pub use event::*;
#[cfg(feature = "app")]
pub use handle::EventHandler;
pub use handle::{ActionHandler, DefaultHandler};
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub use hooks::*;
pub use message::{IntoMessage, Message, MessageAlt, MessageBuild, MessageSegment};
pub use resp::{Resp, RespContent, Resps};
pub use utils::*;

pub use async_trait::async_trait;
