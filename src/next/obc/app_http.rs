use std::{convert::Infallible, sync::Arc};

use crate::{
    action::ActionType,
    config::HttpServer,
    next::{EHACtrait, OneBot, Static},
    utils::ProtocolItem,
    SelfId, WalleError, WalleResult,
};
use hyper::{
    header::AUTHORIZATION, server::conn::Http, service::service_fn, Body, Request, Response,
};
use tokio::{net::TcpListener, task::JoinHandle};
use tracing::{info, warn};

use super::AppOBC;

impl<A, R> AppOBC<A, R>
where
    A: ProtocolItem + ActionType,
    R: ProtocolItem,
{
    async fn http_webhook<E, EHAC, const V: u8>(
        &self,
        ob: &Arc<OneBot<AppOBC<A, R>, EHAC, V>>,
        config: Vec<HttpServer>,
    ) -> WalleResult<Vec<JoinHandle<()>>>
    where
        E: ProtocolItem + SelfId + Clone,
        EHAC: EHACtrait<E, A, R, AppOBC<A, R>, V> + Static,
    {
        let mut tasks = vec![];
        for webhook in config {
            let bot_map = self.bots.clone();
            let access_token = webhook.access_token.clone();
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
                            if bot_map.get(&event.self_id()).is_none() {
                                todo!()
                            }
                            ob.handle_event(event).await
                        }
                        Err(s) => warn!(target: crate::WALLE_CORE, "Webhook json error: {}", s),
                    }
                    Ok::<Response<Body>, Infallible>(Response::new("".into()))
                }
            });
            tasks.push(tokio::spawn(async move {
                loop {
                    let (tcp_stream, _) = listener.accept().await.unwrap();
                    let service = serv.clone();
                    tokio::spawn(async move {
                        Http::new()
                            .serve_connection(tcp_stream, service)
                            .await
                            .unwrap();
                    });
                }
            }));
        }
        Ok(tasks)
    }
}
