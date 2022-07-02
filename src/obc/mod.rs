pub const OBC: &str = "Walle-OBC";

#[cfg(feature = "app-obc")]
mod app_obc;
#[cfg(feature = "impl-obc")]
mod impl_obc;
#[cfg(feature = "websocket")]
mod ws_util;

#[cfg(feature = "app-obc")]
pub use app_obc::*;
#[cfg(feature = "impl-obc")]
pub use impl_obc::*;
