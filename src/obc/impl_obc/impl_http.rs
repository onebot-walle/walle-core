use std::{convert::Infallible, sync::Arc, time::Duration};

use http_body_util::{BodyExt, Full};
use hyper::{
    body::{Buf, Bytes, Incoming},
    header::{AUTHORIZATION, CONTENT_TYPE},
    service::service_fn,
    Method, Request, Response, StatusCode,
};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as ServerAutoBuilder,
};
use tokio::{net::TcpListener, task::JoinHandle};
use tracing::{info, trace, warn};

use crate::{
    config::{HttpClient, HttpServer},
    error::{WalleError, WalleResult},
    resp::{resp_error, Resp},
    util::{AuthReqHeaderExt, ContentType, Echo, ProtocolItem},
    ActionHandler, EventHandler, OneBot,
};

use super::ImplOBC;

type FullBytesResp = Response<Full<Bytes>>;
type HyperClient = Client<HttpConnector, String>;

fn empty_error_response(code: u16) -> FullBytesResp {
    Response::builder()
        .status(code)
        .body(Full::default())
        .unwrap()
}

fn error_response(code: u16, body: &'static str) -> FullBytesResp {
    Response::builder().status(code).body(body.into()).unwrap()
}

fn encode2resp<T: ProtocolItem>(t: T, content_type: &ContentType) -> FullBytesResp {
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
    pub(crate) async fn http<A, R, AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Vec<HttpServer>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        A: ProtocolItem,
        R: ProtocolItem,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        for http in config {
            let ob_ = ob.clone();
            let addr = std::net::SocketAddr::new(http.host, http.port);
            info!(
                target: crate::WALLE_CORE,
                "Starting HTTP server on http://{}", addr
            );
            let access_token = http.access_token.clone();
            let path = http.path.clone();
            let serv = service_fn(move |req: Request<Incoming>| {
                let path = path.clone();
                let access_token = access_token.clone();
                let ob = ob_.clone();
                async move {
                    use crate::obc::check_query;
                    if req.method() != Method::POST {
                        return Ok::<_, Infallible>(empty_error_response(405));
                    }
                    if path
                        .map(|p| req.uri().path() != p)
                        .unwrap_or(!["/", ""].contains(&req.uri().path()))
                    {
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
                        } else if let Some(query_token) = check_query(req.uri()) {
                            if token != query_token {
                                return Ok(error_response(403, "Authorization Query is invalid"));
                            }
                        } else {
                            return Ok(error_response(403, "Missing Authorization Header"));
                        }
                    }
                    let data = req.collect().await.unwrap().to_bytes();
                    let action: Result<Echo<A>, _> = match content_type {
                        ContentType::Json => {
                            ProtocolItem::json_decode(&String::from_utf8(data.to_vec()).unwrap())
                        }
                        ContentType::MsgPack => ProtocolItem::rmp_decode(&data),
                    };
                    match action {
                        Ok(action) => {
                            let (action, echo) = action.unpack();
                            match ob.handle_action(action).await {
                                Ok(r) => Ok(encode2resp(echo.pack(r), &content_type)),
                                Err(e) => {
                                    warn!(target: super::OBC, "handle action error: {}", e);
                                    Ok(encode2resp::<Resp>(
                                        resp_error::bad_handler(e).into(),
                                        &content_type,
                                    ))
                                }
                            }
                        }
                        Err(e) => Ok(encode2resp(
                            if e.starts_with("missing field") {
                                trace!(
                                    target: crate::WALLE_CORE,
                                    "Http call action miss field: {e}",
                                );
                                Resp::from(resp_error::bad_segment_data(e))
                            } else {
                                warn!(target: crate::WALLE_CORE, "Http call action ser error: {e}",);
                                resp_error::unsupported_action(e).into()
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
                                let io = TokioIo::new(tcp_stream);
                                ServerAutoBuilder::new(TokioExecutor::new()).serve_connection(io, serv).await.unwrap();
                            });
                        }
                    }
                }
            }));
        }
        Ok(())
    }

    pub(crate) async fn webhook<A, R, AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Vec<HttpClient>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        E: ProtocolItem + Clone,
        A: ProtocolItem,
        R: ProtocolItem,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        let ob = ob.clone();
        let mut event_rx = self.event_tx.subscribe();
        let mut signal_rx = ob.get_signal_rx()?;
        let r#impl = self.implt.clone();
        let cli: HyperClient = Client::builder(TokioExecutor::new()).build_http::<String>();
        tasks.push(tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = signal_rx.recv() => break,
                    Ok(event) = event_rx.recv() => webhook_push(
                        &ob,
                        event,
                        &r#impl,
                        &config,
                        &cli
                    ).await
                }
            }
        }));
        Ok(())
    }
}

async fn webhook_push<E, A, R, AH, EH>(
    ob: &Arc<OneBot<AH, EH>>,
    event: E,
    r#impl: &str,
    config: &Vec<HttpClient>,
    cli: &HyperClient,
) where
    E: ProtocolItem,
    A: ProtocolItem,
    R: ProtocolItem,
    AH: ActionHandler<E, A, R> + Send + Sync + 'static,
    EH: EventHandler<E, A, R> + Send + Sync + 'static,
{
    let date = event.json_encode();
    for webhook in config {
        let req = Request::builder()
            .method(Method::POST)
            .uri(&webhook.url)
            .header(CONTENT_TYPE, "application/json")
            .header("X-OneBot-Version", 12.to_string())
            .header("X-Impl", r#impl.to_owned())
            .header_auth_token(&webhook.access_token)
            .body(date.clone())
            .unwrap();
        let ob = ob.clone();
        let timeout = webhook.timeout;
        let cli = cli.clone();
        tokio::spawn(async move {
            let resp =
                match tokio::time::timeout(Duration::from_secs(timeout), cli.request(req)).await {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => {
                        warn!(target: crate::WALLE_CORE, "{}", e);
                        return;
                    }
                    Err(_) => {
                        warn!(target: crate::WALLE_CORE, "push event timeout");
                        return;
                    }
                };
            match resp.status() {
                StatusCode::NO_CONTENT => (),
                StatusCode::OK => {
                    let body = resp.collect().await.unwrap().to_bytes();
                    let actions: Vec<A> = match serde_json::from_reader(body.reader()) {
                        Ok(e) => e,
                        Err(_) => {
                            panic!()
                            // handle error here
                        }
                    };
                    for a in actions {
                        let _ = ob.handle_action(a).await;
                    }
                }
                x => info!("unhandle webhook push status: {}", x),
            }
        });
    }
}
