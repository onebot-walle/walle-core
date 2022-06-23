const OBC: &str = "Walle-OBC";

#[cfg(feature = "http")]
mod app_http;
mod app_obc;
#[cfg(feature = "websocket")]
mod app_ws;
mod bot_ext;
#[cfg(feature = "http")]
mod impl_http;
mod impl_obc;
#[cfg(feature = "websocket")]
mod impl_ws;
#[cfg(feature = "websocket")]
mod ws_util;

pub use app_obc::*;
pub use impl_obc::*;
