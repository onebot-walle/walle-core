use std::{convert::Infallible, sync::Arc};

use hyper::header::AUTHORIZATION;
use hyper::server::conn::Http;
use hyper::service::service_fn;
use hyper::{Body, Request, Response};
use tokio::net::TcpListener;
use tracing::info;

use crate::utils::ProtocolItem;
use crate::{app::OneBot, handle::EventHandler, SelfId, WalleError, WalleResult};

impl<E, A, R, H, const V: u8> OneBot<E, A, R, H, V>
where
    E: ProtocolItem + Send + Clone + SelfId + 'static,
    A: ProtocolItem + Send + 'static,
    R: ProtocolItem + Send + 'static,
    H: EventHandler<E, A, R> + Send + Sync + 'static,
{
    #[allow(dead_code)]
    pub(crate) async fn http_webhook(self: &Arc<Self>) -> WalleResult<()> {
        for webhook in &self.config.http_webhook {
            let ob = self.clone();
            let access_token = webhook.access_token.clone();
            let addr = std::net::SocketAddr::new(webhook.host, webhook.port);
            info!(
                target: crate::WALLE_CORE,
                "Starting HTTP Webhook server on http://{}", addr
            );
            let listener = TcpListener::bind(&addr).await.map_err(WalleError::from)?;
            let serv = service_fn(move |req: Request<Body>| {
                let access_token = access_token.clone();
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
                    //todo
                    Ok::<Response<Body>, Infallible>(Response::new("".into()))
                }
            });
            tokio::spawn(async move {
                while ob.is_running() {
                    let (tcp_stream, _) = listener.accept().await.unwrap();
                    let service = serv.clone();
                    tokio::spawn(async move {
                        Http::new()
                            .serve_connection(tcp_stream, service)
                            .await
                            .unwrap();
                    });
                }
            });
            self.set_running();
        }
        Ok(())
    }
}
