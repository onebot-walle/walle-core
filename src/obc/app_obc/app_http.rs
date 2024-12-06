use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};

use crate::{
    config::{HttpClient, HttpServer},
    error::{WalleError, WalleResult},
    prelude::Bot,
    structs::Selft,
    util::{AuthReqHeaderExt, Echo, GetSelf, ProtocolItem},
    ActionHandler, EventHandler, OneBot,
};
use http_body_util::BodyExt;
use hyper::{
    body::{Buf, Incoming},
    header::{AUTHORIZATION, CONTENT_TYPE},
    service::service_fn,
    Method, Request, Response,
};
use hyper_util::{
    client::legacy::{connect::HttpConnector, Client},
    rt::{TokioExecutor, TokioIo},
    server::conn::auto::Builder as ServerAutoBuilder,
};
use tokio::{net::TcpListener, task::JoinHandle};
use tracing::{info, warn};

use super::{AppOBC, EchoMap};

impl<A, R> AppOBC<A, R>
where
    A: ProtocolItem,
    R: ProtocolItem,
{
    pub(crate) async fn webhook<E, AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: Vec<HttpServer>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        E: ProtocolItem + GetSelf + Clone,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        for webhook in config {
            let echo_map = self.echos.clone();
            let path = webhook.path.clone();
            let access_token = webhook.access_token.clone();
            let mut signal_rx = ob.get_signal_rx()?;
            let ob = ob.clone();
            let addr = std::net::SocketAddr::new(webhook.host, webhook.port);
            info!(
                target: crate::WALLE_CORE,
                "Starting HTTP Webhook server on http://{}", addr
            );
            let listener = TcpListener::bind(&addr).await.map_err(WalleError::from)?;
            let map = self.get_bot_map().clone();
            let serv = service_fn(move |req: Request<Incoming>| {
                let path = path.clone();
                let access_token = access_token.clone();
                let ob = ob.clone();
                let echo_map = echo_map.clone();
                let map = map.clone();
                async move {
                    if path
                        .map(|p| req.uri().path() != p)
                        .unwrap_or(!["/", ""].contains(&req.uri().path()))
                    {
                        return Ok(Response::builder()
                            .status(404)
                            .body("Not Found".into())
                            .unwrap());
                    }
                    if let Some(token) = access_token.as_ref() {
                        if let Some(header_token) = req
                            .headers()
                            .get(AUTHORIZATION)
                            .and_then(|v| v.to_str().ok())
                        {
                            if header_token != format!("Bearer {}", token) {
                                return Ok(Response::builder()
                                    .status(403)
                                    .body("Authorization Header is invalid".into())
                                    .unwrap());
                            }
                        } else {
                            return Ok(Response::builder()
                                .status(403)
                                .body("Missing Authorization Header".into())
                                .unwrap());
                        }
                    }
                    let implt = req
                        .headers()
                        .get("X-Impl")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_owned())
                        .unwrap_or_default();
                    let body = String::from_utf8(req.collect().await.unwrap().to_bytes().to_vec())
                        .unwrap();
                    match E::json_decode(&body) {
                        Ok(event) => {
                            let (seq, mut action_rx) = map.new_connect();
                            let selft = event.get_self();
                            map.connect_update(
                                &seq,
                                vec![Bot {
                                    online: true,
                                    selft,
                                }],
                                &implt,
                            );
                            if let Err(e) = ob.handle_event(event).await {
                                warn!(target: super::OBC, "{}", e);
                            }
                            if let Ok(Some(a)) = tokio::time::timeout(
                                std::time::Duration::from_secs(8),
                                action_rx.recv(),
                            )
                            .await
                            {
                                let echo_s = a.get_echo();
                                echo_map.remove(&echo_s);
                                map.connect_closs(&seq);
                                return Ok(Response::new(a.json_encode().into()));
                            }
                        }
                        Err(s) => warn!(target: crate::WALLE_CORE, "Webhook json error: {}", s),
                    }
                    Ok::<Response<String>, Infallible>(Response::new("".into()))
                }
            });
            tasks.push(tokio::spawn(async move {
                loop {
                    let service = serv.clone();
                    tokio::select! {
                        _ = signal_rx.recv() => break,
                        Ok((tcp_stream, _)) = listener.accept() => {
                            tokio::spawn(async move {
                                let io = TokioIo::new(tcp_stream);
                                ServerAutoBuilder::new(TokioExecutor::new())
                                    .serve_connection(io, service)
                                    .await
                                    .unwrap();
                            });
                        }
                    }
                }
            }));
        }
        Ok(())
    }

    pub(crate) async fn http<E, AH, EH>(
        &self,
        ob: &Arc<OneBot<AH, EH>>,
        config: HashMap<String, HttpClient>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        E: ProtocolItem + GetSelf + Clone,
        AH: ActionHandler<E, A, R> + Send + Sync + 'static,
        EH: EventHandler<E, A, R> + Send + Sync + 'static,
    {
        for (bot_id, http) in config {
            let (seq, mut rx) = self.get_bot_map().new_connect();
            let implt = http.implt.clone().unwrap_or_default();
            self.get_bot_map().connect_update(
                &seq,
                vec![Bot {
                    online: true,
                    selft: Selft {
                        platform: http.platform.clone().unwrap_or_default(),
                        user_id: bot_id.clone(),
                    },
                }],
                &implt,
            );
            let ob = ob.clone();
            let echo_map = self.echos.clone();
            let mut signal_rx = ob.get_signal_rx()?;
            let cli = Client::builder(TokioExecutor::new()).build_http::<String>();
            tasks.push(tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = signal_rx.recv() => break,
                        Some(action) = rx.recv() => {
                            tokio::spawn(http_push(
                                action,
                                http.clone(),
                                echo_map.clone(),
                                cli.clone(),
                            ));
                        }
                    }
                }
            }));
        }
        Ok(())
    }
}

async fn http_push<A, R>(
    action: Echo<A>,
    http: HttpClient,
    echo_map: EchoMap<R>,
    cli: Client<HttpConnector, String>,
) where
    A: ProtocolItem,
    R: ProtocolItem,
{
    let (action, echo_s) = action.unpack();
    let req = Request::builder()
        .method(Method::POST)
        .uri(&http.url)
        .header_auth_token(&http.access_token)
        .header(CONTENT_TYPE, crate::util::ContentType::Json.to_string())
        .body(action.json_encode()) //todo
        .unwrap();
    match tokio::time::timeout(Duration::from_secs(http.timeout), cli.request(req)).await {
        Ok(Ok(resp)) => {
            let body = resp.collect().await.unwrap().to_bytes();
            let r: R = serde_json::from_reader(body.reader()).unwrap();
            if let Some((_, r_tx)) = echo_map.remove(&echo_s) {
                r_tx.send(r).ok();
            }
        }
        Ok(Err(e)) => {
            warn!(target: crate::WALLE_CORE, "HTTP push error: {}", e);
        }
        Err(e) => {
            warn!(target: crate::WALLE_CORE, "HTTP push timeout: {}", e);
        }
    }
}
