pub(crate) trait AuthReqHeaderExt {
    fn header_auth_token(self, token: &Option<String>) -> Self;
}

#[cfg(feature = "http")]
use hyper::http::{header::AUTHORIZATION, request::Builder};
#[cfg(all(feature = "websocket", not(feature = "http")))]
use tokio_tungstenite::tungstenite::http::{header::AUTHORIZATION, request::Builder};

#[cfg(any(feature = "websocket", feature = "http"))]
impl AuthReqHeaderExt for Builder {
    fn header_auth_token(self, token: &Option<String>) -> Self {
        if let Some(token) = token {
            self.header(AUTHORIZATION, format!("Bearer {}", token))
        } else {
            self
        }
    }
}
