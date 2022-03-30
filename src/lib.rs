#![cfg_attr(docsrs, feature(doc_cfg))]
#![doc = include_str!("../README.md")]

#[allow(dead_code)]
const VERSION: &str = "0.1.1";

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

use serde::{Deserialize, Serialize};
pub use utils::ColoredAlt;

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
pub use message::{Message, MessageAlt, MessageBuild, MessageSegment};
pub use resp::{Resp, RespContent, Resps};
pub use utils::*;

pub use async_trait::async_trait;

pub trait ProtocolItem: Serialize + for<'de> Deserialize<'de> {
    fn json_encode(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
    fn json_decode(s: &str) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::from_str(s)
    }
    fn json_fron_reader<R: std::io::Read>(rdr: R) -> Result<Self, serde_json::Error>
    where
        Self: Sized,
    {
        serde_json::from_reader(rdr)
    }
    fn rmp_encode(&self) -> Vec<u8> {
        rmp_serde::to_vec(self).unwrap()
    }
    fn rmp_decode(v: &[u8]) -> Result<Self, rmp_serde::decode::Error>
    where
        Self: Sized,
    {
        rmp_serde::from_slice(v)
    }
}

impl<T> ProtocolItem for T where T: Serialize + for<'de> Deserialize<'de> {}
