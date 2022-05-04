use hyper::body::Buf;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use hyper::service::Service;
use hyper::Method;
use hyper::{server::conn::Http, Body, Request, Response};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::net::TcpListener;
use tracing::info;

use crate::comms::utils::ContentType;
use crate::impls::CustomOneBot;
use crate::utils::Echo;
use crate::Resps;
use crate::{ProtocolItem, WalleError, WalleResult};

fn empty_error_response(code: u16) -> Response<Body> {
    Response::builder()
        .status(code)
        .body(Body::empty())
        .unwrap()
}

fn error_response(code: u16, body: &'static str) -> Response<Body> {
    Response::builder().status(code).body(body.into()).unwrap()
}

#[derive(Clone)]
struct OneBotService<E, A, R, const V: u8> {
    pub access_token: Option<String>,
    pub ob: Arc<crate::impls::CustomOneBot<E, A, R, V>>,
}

impl<E, A, R, const V: u8> Service<Request<Body>> for OneBotService<E, A, R, V>
where
    E: Send + 'static,
    A: ProtocolItem + std::fmt::Debug + Send + 'static,
    R: ProtocolItem + std::fmt::Debug + Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let access_token = self.access_token.clone();
        let ob = self.ob.clone();
        Box::pin(async move {
            if req.method() != Method::POST {
                return Ok(empty_error_response(405));
            }
            if req.uri() != "/" {
                return Ok(empty_error_response(404));
            }
            let content_type = match req
                .headers()
                .get(CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| ContentType::new(s))
            {
                Some(t) => t,
                None => return Ok(empty_error_response(415)),
            };

            if let Some(ref token) = access_token {
                if let Some(header_token) = req
                    .headers()
                    .get(AUTHORIZATION)
                    .and_then(|a| a.to_str().ok())
                {
                    if header_token != format!("Bearer {}", token).as_str() {
                        return Ok(error_response(403, "Authorization Header is invalid"));
                    }
                } else {
                    return Ok(error_response(403, "Missing Authorization Header"));
                }
            }
            let data = hyper::body::aggregate(req).await.unwrap();
            let action: Result<Echo<A>, _> = match content_type {
                ContentType::Json => ProtocolItem::json_from_reader(data.reader()),
                ContentType::MsgPack => ProtocolItem::rmp_from_reader(data.reader()),
            };
            match action {
                Ok(action) => {
                    let (action, echo) = action.unpack();
                    let action_resp = ob.action_handler.handle(action, &ob).await;
                    Ok(encode2resp(echo.pack(action_resp), &content_type))
                }
                Err(e) => Ok(encode2resp(
                    if e.starts_with("missing field") {
                        Resps::empty_fail(10006, e)
                    } else {
                        Resps::unsupported_action()
                    },
                    &content_type,
                )),
            }
        })
    }
}

fn encode2resp<T: ProtocolItem>(t: T, content_type: &ContentType) -> Response<Body> {
    match content_type {
        ContentType::Json => Response::builder()
            .header(CONTENT_TYPE, "application/json")
            .body(t.json_encode().into())
            .unwrap(),
        ContentType::MsgPack => Response::builder()
            .header(CONTENT_TYPE, "application/msgpack")
            .body(t.rmp_encode().into())
            .unwrap(),
    }
}

impl<E, A, R, const V: u8> CustomOneBot<E, A, R, V>
where
    E: Clone + Send + 'static,
    A: ProtocolItem + std::fmt::Debug + Clone + Send + 'static,
    R: ProtocolItem + std::fmt::Debug + Clone + Send + 'static,
{
    pub(crate) async fn http(self: &Arc<Self>) -> WalleResult<()> {
        for http in &self.config.http {
            let ob = self.clone();
            let addr = std::net::SocketAddr::new(http.host, http.port);
            info!(target: "Walle-core", "Starting HTTP server on http://{}", addr);
            let access_token = http.access_token.clone();
            let listener = TcpListener::bind(&addr).await.map_err(WalleError::from)?;
            tokio::spawn(async move {
                let serv = OneBotService {
                    access_token,
                    ob: ob.clone(),
                };
                while ob.is_running() {
                    let (tcp_stream, _) = listener.accept().await.unwrap();
                    let serv = serv.clone();
                    tokio::spawn(async move {
                        Http::new()
                            .serve_connection(tcp_stream, serv)
                            .await
                            .unwrap(); //Infallible
                    });
                }
            });
            self.set_running();
        }
        Ok(())
    }
}
