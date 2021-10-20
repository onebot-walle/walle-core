use hyper::Method;
use hyper::{service::Service, Body, Request, Response, Server};
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

struct OneBotService(Option<String>);

impl Service<Request<Body>> for OneBotService {
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

        if let (Some(token), Some(header_token)) = (&self.0, req.headers().get("Authorization")) {
            let header_token = header_token.to_str().unwrap();
            if header_token != &format!("Bearer {}", token) {
                return Box::pin(async { Ok(empty_error_response(401)) });
            }
        }
        Box::pin(async { Ok(Response::new(Body::empty())) })
    }
}

struct MakeOneBotService(Option<String>);

impl<T> Service<T> for MakeOneBotService {
    type Response = OneBotService;
    type Error = Infallible;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, _: T) -> Self::Future {
        let obs = OneBotService(self.0.clone());
        Box::pin(async move { Ok(obs) })
    }
}

pub async fn run(config: &crate::config::Http) -> tokio::task::JoinHandle<()> {
    let mobs = MakeOneBotService(config.access_token.clone());
    let addr = std::net::SocketAddr::new(config.host, config.port);
    let server = Server::bind(&addr).serve(mobs);
    tokio::spawn(async { server.await.unwrap() })
}
