use std::{
    collections::{HashMap, HashSet},
    convert::Infallible,
    sync::Arc,
    time::Duration,
};

use crate::{
    config::{HttpClient, HttpServer},
    error::{WalleError, WalleResult},
    structs::Selft,
    util::{AuthReqHeaderExt, Echo, GetSelf, ProtocolItem},
    ActionHandler, EventHandler, OneBot,
};
use hyper::{
    body::Buf,
    client::HttpConnector,
    header::{AUTHORIZATION, CONTENT_TYPE},
    server::conn::Http,
    service::service_fn,
    Body, Client as HyperClient, Method, Request, Response,
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
                    let implt = req
                        .headers()
                        .get("X-Impl")
                        .and_then(|v| v.to_str().ok())
                        .map(|s| s.to_owned())
                        .unwrap_or_default();
                    let body = String::from_utf8(
                        hyper::body::to_bytes(req.into_body())
                            .await
                            .unwrap()
                            .to_vec(),
                    )
                    .unwrap();
                    match E::json_decode(&body) {
                        Ok(event) => {
                            let (seq, mut action_rx) = bot_map.new_connect();
                            let selft = event.get_self();
                            bot_map.connect_update(&seq, HashSet::from([selft]), &implt);
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
                                bot_map.connect_closs(&seq);
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
        let client = Arc::new(HyperClient::new());
        for (bot_id, http) in config {
            let (seq, mut rx) = self.bots.new_connect();
            let implt = http.implt.clone().unwrap_or_default();
            // let (tx, mut rx) = mpsc::unbounded_channel();
            self.bots.connect_update(
                &seq,
                HashSet::from([Selft {
                    platform: http.platform.clone().unwrap_or_default(),
                    user_id: bot_id.clone(),
                }]),
                &implt,
            );
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
    A: ProtocolItem,
    R: ProtocolItem,
{
    let (action, echo_s) = action.unpack();
    let req = Request::builder()
        .method(Method::POST)
        .uri(&http.url)
        .header_auth_token(&http.access_token)
        .header(CONTENT_TYPE, crate::util::ContentType::Json.to_string())
        .body(action.to_body(&crate::util::ContentType::Json)) //todo
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
