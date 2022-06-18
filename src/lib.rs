#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[allow(dead_code)]
const VERSION: &str = std::env!("CARGO_PKG_VERSION");
const WALLE_CORE: &str = "Walle-core";

mod comms;
mod error;
mod handle;
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
mod hooks;
mod test;
mod utils;

/// Onebot Action
pub mod action;
#[cfg(feature = "app")]
#[cfg_attr(docsrs, doc(cfg(feature = "app")))]
/// 应用端相关 api
pub mod app;
/// 相关配置项
pub mod config;
/// Onebot Event
pub mod event;
#[cfg(feature = "impl")]
#[cfg_attr(docsrs, doc(cfg(feature = "impl")))]
/// 实现端相关 api
pub mod impls;
pub mod message;
/// Onebot ActionResp
pub mod resp;

pub use action::StandardAction;
pub use error::*;
pub use event::*;
#[cfg(feature = "app")]
pub use handle::EventHandler;
pub use handle::{ActionHandler, DefaultHandler};
#[cfg(feature = "websocket")]
#[cfg_attr(docsrs, doc(cfg(feature = "websocket")))]
pub use hooks::*;
pub use message::{IntoMessage, Message, MessageAlt, MessageBuild, MessageSegment};
pub use resp::{Resp, RespContent, Resps, StandardResps};
#[cfg(all(feature = "websocket", feature = "impl"))]
#[cfg_attr(docsrs, doc(cfg(all(feature = "websocket", feature = "impl"))))]
pub use utils::timestamp_nano_f64;
pub use utils::{ColoredAlt, ContentType, ExtendedMap, ExtendedMapExt, ExtendedValue, SelfId};

pub use async_trait::async_trait;

pub mod onebot;
