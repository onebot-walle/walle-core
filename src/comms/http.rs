use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Request, Response, Server};
use std::convert::Infallible;

pub async fn run(addr: std::net::SocketAddr) -> tokio::task::JoinHandle<()> {
    let make_svc = make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_conn)) });
    let server = Server::bind(&addr).serve(make_svc);
    tokio::spawn(async { server.await.unwrap() })
}

async fn handle_conn(_req: Request<Body>) -> Result<Response<Body>, Infallible> {
    Ok(Response::new("todo".into()))
    // todo
}
