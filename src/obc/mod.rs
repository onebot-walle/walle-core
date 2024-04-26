//! OBC

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

#[cfg(feature = "http")]
use hyper::Uri;
#[cfg(all(not(feature = "http"), feature = "websocket"))]
use tokio_tungstenite::tungstenite::http::Uri;

fn check_query(uri: &Uri) -> Option<&str> {
    uri.query()
        .unwrap_or_default()
        .split('&')
        .map(|v| v.split_once('='))
        .collect::<Option<std::collections::HashMap<&str, &str>>>()
        .unwrap_or_default()
        .get("access_token")
        .cloned()
}

#[test]
fn test_query() {
    println!("{:?}", check_query(&"/?key=v&a=b".parse::<Uri>().unwrap()));
    println!(
        "{:?}",
        check_query(&"/?access_token=v&a=b".parse::<Uri>().unwrap())
    )
}
