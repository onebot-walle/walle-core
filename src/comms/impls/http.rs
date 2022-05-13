use hyper::body::Buf;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use hyper::service::service_fn;
use hyper::Method;
use hyper::{server::conn::Http, Body, Request, Response};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;

use crate::comms::utils::ContentType;
use crate::handle::ActionHandler;
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

impl<E, A, R, H, const V: u8> CustomOneBot<E, A, R, H, V>
where
    E: Clone + Send + 'static,
    A: ProtocolItem + std::fmt::Debug + Clone + Send + 'static,
    R: ProtocolItem + std::fmt::Debug + Clone + Send + 'static,
    H: ActionHandler<A, R, Self> + Send + Sync + 'static,
{
    pub(crate) async fn http(self: &Arc<Self>) -> WalleResult<()> {
        for http in &self.config.http {
            let ob = self.clone();
            let addr = std::net::SocketAddr::new(http.host, http.port);
            info!(target: "Walle-core", "Starting HTTP server on http://{}", addr);
            let access_token = http.access_token.clone();
            let serv = service_fn(move |req: Request<Body>| {
                let access_token = access_token.clone();
                let ob = ob.clone();
                async move {
                    if req.method() != Method::POST {
                        return Ok::<Response<Body>, Infallible>(empty_error_response(405));
                    }
                    if req.uri() != "/" {
                        return Ok(empty_error_response(404));
                    }
                    let content_type = match req
                        .headers()
                        .get(CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .and_then(ContentType::new)
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
                }
            });
            let ob = self.clone();
            let listener = TcpListener::bind(&addr).await.map_err(WalleError::from)?;
            tokio::spawn(async move {
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
