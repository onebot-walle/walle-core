use hyper::body::Buf;
use hyper::Method;
use hyper::{service::Service, Body, Request, Response, Server};
use serde::{de::DeserializeOwned, Serialize};
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

fn empty_error_response(code: u16) -> Response<Body> {
    Response::builder()
        .status(code)
        .body(Body::empty())
        .unwrap()
}

#[cfg(feature = "impl")]
struct OneBotService<A, R> {
    pub access_token: Option<String>,
    pub sender: crate::impls::CustomActionSender<A, R>,
}

#[cfg(feature = "impl")]
impl<A, R> Service<Request<Body>> for OneBotService<A, R>
where
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
            .get("Content-Type")
            .and_then(|t| super::util::ContentTpye::new(t.to_str().unwrap()))
        {
            Some(t) => t,
            None => return Box::pin(async { Ok(empty_error_response(415)) }),
        };

        if let (Some(token), Some(header_token)) =
            (&self.access_token, req.headers().get("Authorization"))
        {
            let header_token = header_token.to_str().unwrap();
            if header_token != &format!("Bearer {}", token) {
                return Box::pin(async { Ok(empty_error_response(401)) });
            }
        }
        let (sender, receiver) = tokio::sync::oneshot::channel();
        let action_sender = self.sender.clone();
        Box::pin(async move {
            let data = hyper::body::aggregate(req).await.unwrap();
            let action = serde_json::from_reader(data.reader()).unwrap();
            action_sender
                .send((action, crate::impls::CustomARSS::OneShot(sender)))
                .await
                .unwrap();
            let action_resp = receiver.await.unwrap();
            Ok(Response::new(Body::from(
                serde_json::to_string(&action_resp).unwrap(),
            )))
        })
    }
}

#[cfg(feature = "impl")]
struct MakeOneBotService<A, R> {
    pub access_token: Option<String>,
    pub sender: crate::impls::CustomActionSender<A, R>,
}

#[cfg(feature = "impl")]
impl<T, A, R> Service<T> for MakeOneBotService<A, R>
where
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    type Response = OneBotService<A, R>;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let obs = OneBotService {
            access_token: self.access_token.clone(),
            sender: self.sender.clone(),
        };
        Box::pin(async move { Ok(obs) })
    }
}

#[cfg(feature = "impl")]
pub fn run<A, R>(
    config: &crate::config::Http,
    sender: crate::impls::CustomActionSender<A, R>,
) -> tokio::task::JoinHandle<()>
where
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    let mobs = MakeOneBotService {
        access_token: config.access_token.clone(),
        sender,
    };
    let addr = std::net::SocketAddr::new(config.host, config.port);
    let server = Server::bind(&addr).serve(mobs);
    tokio::spawn(async { server.await.unwrap() })
}
