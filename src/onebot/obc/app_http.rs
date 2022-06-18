use std::{collections::HashMap, convert::Infallible, sync::Arc, time::Duration};

use crate::{
    action::ActionType,
    comms::utils::AuthReqHeaderExt,
    config::{HttpClient, HttpServer},
    onebot::{EventHandler, OneBotExt, Static},
    utils::{Echo, ProtocolItem},
    SelfId, WalleError, WalleResult,
};
use hyper::{
    body::Buf,
    client::HttpConnector,
    header::{AUTHORIZATION, CONTENT_TYPE},
    server::conn::Http,
    service::service_fn,
    Body, Client as HyperClient, Method, Request, Response,
};
use tokio::{net::TcpListener, sync::mpsc, task::JoinHandle};
use tracing::{info, warn};

use super::{AppOBC, EchoMap};

impl<A, R> AppOBC<A, R>
where
    A: ProtocolItem + ActionType,
    R: ProtocolItem,
{
    pub(crate) async fn webhook<E, OB>(
        &self,
        ob: &Arc<OB>,
        config: Vec<HttpServer>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        E: ProtocolItem + SelfId + Clone,
        OB: EventHandler<E, A, R, OB> + OneBotExt + Static,
    {
        for webhook in config {
            let bot_map = self.bots.clone();
            let echo_map = self.echos.clone();
            let access_token = webhook.access_token.clone();
            let mut signal_rx = ob.get_signal_rx()?;
            let ob = ob.clone();
            let addr = std::net::SocketAddr::new(webhook.host, webhook.port);
            info!(
                target: crate::WALLE_CORE,
                "Starting HTTP Webhook server on http://{}", addr
            );
            let listener = TcpListener::bind(&addr).await.map_err(WalleError::from)?;
            let serv = service_fn(move |req: Request<Body>| {
                let access_token = access_token.clone();
                let ob = ob.clone();
                let bot_map = bot_map.clone();
                let echo_map = echo_map.clone();
                async move {
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
                    let body = String::from_utf8(
                        hyper::body::to_bytes(req.into_body())
                            .await
                            .unwrap()
                            .to_vec(),
                    )
                    .unwrap();
                    match E::json_decode(&body) {
                        Ok(event) => {
                            let (action_tx, mut action_rx) = mpsc::unbounded_channel();
                            let self_id = event.self_id();
                            bot_map.ensure_tx(&self_id, &action_tx);
                            ob.handle_event(event, &ob).await;
                            if let Ok(Some(a)) = tokio::time::timeout(
                                std::time::Duration::from_secs(8),
                                action_rx.recv(),
                            )
                            .await
                            {
                                let echo_s = a.get_echo();
                                echo_map.remove(&echo_s);
                                bot_map.remove_bot(&self_id, &action_tx);
                                return Ok(Response::new(a.json_encode().into()));
                            }
                        }
                        Err(s) => warn!(target: crate::WALLE_CORE, "Webhook json error: {}", s),
                    }
                    Ok::<Response<Body>, Infallible>(Response::new("".into()))
                }
            });
            tasks.push(tokio::spawn(async move {
                loop {
                    let service = serv.clone();
                    tokio::select! {
                        _ = signal_rx.recv() => break,
                        Ok((tcp_stream, _)) = listener.accept() => {
                            tokio::spawn(async move {
                                Http::new()
                                    .serve_connection(tcp_stream, service)
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

    pub(crate) async fn http<E, OB>(
        &self,
        ob: &Arc<OB>,
        config: HashMap<String, HttpClient>,
        tasks: &mut Vec<JoinHandle<()>>,
    ) -> WalleResult<()>
    where
        E: ProtocolItem + SelfId + Clone,
        OB: EventHandler<E, A, R, OB> + OneBotExt + Static,
    {
        let client = Arc::new(HyperClient::new());
        for (bot_id, http) in config {
            let (tx, mut rx) = mpsc::unbounded_channel();
            self.bots.ensure_tx(&bot_id, &tx);
            let ob = ob.clone();
            let cli = client.clone();
            let echo_map = self.echos.clone();
            let mut signal_rx = ob.get_signal_rx()?;
            tasks.push(tokio::spawn(async move {
                loop {
                    tokio::select! {
                        _ = signal_rx.recv() => break,
                        Some(action) = rx.recv() => {
                            tokio::spawn(http_push(
                                action,
                                cli.clone(),
                                http.clone(),
                                echo_map.clone(),
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
    client: Arc<HyperClient<HttpConnector, Body>>,
    http: HttpClient,
    echo_map: EchoMap<R>,
) where
    A: ProtocolItem + ActionType,
    R: ProtocolItem,
{
    let (action, echo_s) = action.unpack();
    let req = Request::builder()
        .method(Method::POST)
        .uri(&http.url)
        .header_auth_token(&http.access_token)
        .header(CONTENT_TYPE, action.content_type().to_string())
        .body(action.to_body())
        .unwrap();
    match tokio::time::timeout(Duration::from_secs(http.timeout), client.request(req)).await {
        Ok(Ok(resp)) => {
            let body = hyper::body::aggregate(resp).await.unwrap(); //todo
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
