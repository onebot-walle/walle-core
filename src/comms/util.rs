#[cfg(any(feature = "http", feature = "websocket"))]
#[allow(dead_code)]
pub enum ContentTpye {
    Json,
    MsgPack,
}

#[cfg(any(feature = "http", feature = "websocket"))]
impl ContentTpye {
    #[allow(dead_code)]
    pub fn new(s: &str) -> Option<Self> {
        match s {
            "application/json" => Some(Self::Json),
            "application/msgpack" => Some(Self::MsgPack),
            _ => None,
        }
    }
}

pub(crate) trait AuthReqHeaderExt {
    fn header_auth_token(self, token: &Option<String>) -> Self;
}

#[cfg(feature = "http")]
use hyper::http::request::Builder;
#[cfg(all(feature = "websocket", not(feature = "http")))]
use tokio_tungstenite::tungstenite::http::request::Builder;


impl AuthReqHeaderExt for Builder {
    fn header_auth_token(self, token: &Option<String>) -> Self {
        if let Some(token) = token {
            self.header("Authorization", format!("Bearer {}", token))
        } else {
            self
        }
    }
}
