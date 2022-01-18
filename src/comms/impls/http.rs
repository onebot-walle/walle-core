use hyper::body::Buf;
use hyper::header::{AUTHORIZATION, CONTENT_TYPE};
use hyper::service::Service;
use hyper::Method;
use hyper::{server::conn::Http, Body, Request, Response};
use serde::{de::DeserializeOwned, Serialize};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::net::TcpListener;

use crate::impls::CustomOneBot;
use crate::utils::Echo;
use crate::{WalleError, WalleResult};

fn empty_error_response(code: u16) -> Response<Body> {
    Response::builder()
        .status(code)
        .body(Body::empty())
        .unwrap()
}

#[derive(Clone)]
struct OneBotService<E, A, R, const V: u8> {
    pub access_token: Option<String>,
    pub ob: Arc<crate::impls::CustomOneBot<E, A, R, V>>,
}

impl<E, A, R, const V: u8> Service<Request<Body>> for OneBotService<E, A, R, V>
where
    E: Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        if req.method() != Method::POST {
            return Box::pin(async { Ok(empty_error_response(405)) });
        }
        if req.uri() != "/" {
            return Box::pin(async { Ok(empty_error_response(404)) });
        }
        let _content_type = match &req
            .headers()
            .get(CONTENT_TYPE)
            .and_then(|t| crate::comms::util::ContentTpye::new(t.to_str().unwrap()))
        {
            Some(t) => t,
            None => return Box::pin(async { Ok(empty_error_response(415)) }),
        };

        if let (Some(token), Some(header_token)) =
            (&self.access_token, req.headers().get(AUTHORIZATION))
        {
            let header_token = header_token.to_str().unwrap();
            if header_token != format!("Bearer {}", token).as_str() {
                return Box::pin(async { Ok(empty_error_response(401)) });
            }
        }
        let ob = self.ob.clone();
        Box::pin(async move {
            let data = hyper::body::aggregate(req).await.unwrap();
            let action: Echo<A> = serde_json::from_reader(data.reader()).unwrap();
            let (action, echo) = action.unpack();
            let action_resp = ob.action_handler.handle(action, &ob).await;
            let action_resp = echo.pack(action_resp);
            Ok(Response::new(Body::from(
                serde_json::to_string(&action_resp).unwrap(),
            )))
        })
    }
}

impl<E, A, R, const V: u8> CustomOneBot<E, A, R, V>
where
    E: Clone + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Clone + Send + 'static,
    R: Serialize + std::fmt::Debug + Clone + Send + 'static,
{
    pub(crate) async fn http(self: &Arc<Self>) -> WalleResult<()> {
        for http in &self.config.http {
            let ob = self.clone();
            let addr = std::net::SocketAddr::new(http.host, http.port);
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
                            .unwrap();
                    });

                    //todo
                }
            });
            self.set_running();
        }
        Ok(())
    }
}
