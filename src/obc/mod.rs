pub const OBC: &str = "Walle-OBC";

#[cfg(feature = "app_obc")]
mod app_obc;
#[cfg(feature = "impl_obc")]
mod impl_obc;
#[cfg(feature = "websocket")]
mod ws_util;

#[cfg(feature = "app_obc")]
pub use app_obc::*;
#[cfg(feature = "impl_obc")]
pub use impl_obc::*;
