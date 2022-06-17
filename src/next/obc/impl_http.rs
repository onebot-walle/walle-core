use std::{convert::Infallible, sync::Arc};

use hyper::{
    header::{AUTHORIZATION, CONTENT_TYPE},
    server::conn::Http,
    service::service_fn,
    Body, Method, Request, Response,
};
use tokio::{net::TcpListener, task::JoinHandle};
use tracing::{info, trace, warn};

use crate::{
    config::HttpServer,
    next::{ActionHandler, OneBotExt, Static},
    resp::error_builder,
    utils::{Echo, ProtocolItem},
    ContentType, Resps, WalleError, WalleResult,
};

use super::ImplOBC;

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

impl<E> ImplOBC<E>
where
    E: ProtocolItem + Clone,
{
    pub(crate) async fn http<A, R, OB>(
        &self,
        ob: &Arc<OB>,
        config: Vec<HttpServer>,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        A: ProtocolItem,
        R: ProtocolItem,
        OB: ActionHandler<E, A, R, OB> + OneBotExt + Static,
    {
        let mut tasks = vec![];
        for http in config {
            let ob = ob.clone();
            let addr = std::net::SocketAddr::new(http.host, http.port);
            info!(
                target: crate::WALLE_CORE,
                "Starting HTTP server on http://{}", addr
            );
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
                    let data = hyper::body::to_bytes(req).await.unwrap();
                    let action: Result<Echo<A>, _> = match content_type {
                        ContentType::Json => {
                            ProtocolItem::json_decode(&String::from_utf8(data.to_vec()).unwrap())
                        }
                        ContentType::MsgPack => ProtocolItem::rmp_decode(&data),
                    };
                    match action {
                        Ok(action) => {
                            let (action, echo) = action.unpack();
                            let action_resp = ob.handle_action(action, &ob).await;
                            Ok(encode2resp(echo.pack(action_resp), &content_type))
                        }
                        Err(e) => Ok(encode2resp(
                            if e.starts_with("missing field") {
                                trace!(
                                    target: crate::WALLE_CORE,
                                    "Http call action miss field: {e}",
                                );
                                Resps::<E>::empty_fail(10006, e)
                            } else {
                                warn!(target: crate::WALLE_CORE, "Http call action ser error: {e}",);
                                error_builder::unsupported_action().into()
                            },
                            &content_type,
                        )),
                    }
                }
            });
            let ob = ob.clone();
            let listener = TcpListener::bind(&addr).await.map_err(WalleError::from)?;
            let mut signal_rx = ob.get_signal_rx()?;
            tasks.push(tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = signal_rx.recv() => break,
                        Ok((tcp_stream, _)) = listener.accept() => {
                            let serv = serv.clone();
                            tokio::spawn(async move {
                                Http::new()
                                    .serve_connection(tcp_stream, serv)
                                    .await
                                    .unwrap(); //Infallible
                            });
                        }
                    }
                }
            }));
        }
        Ok(tasks)
    }
}
