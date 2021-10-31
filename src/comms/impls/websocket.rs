use super::util::WebSocketServer;
use crate::config::WebSocket;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;

pub async fn run<E, A, R>(
    websocket: &WebSocket,
    broadcaster: crate::impls::CustomEventBroadcaster<E>,
    sender: crate::impls::CustomActionSender<A, R>,
) -> WebSocketServer
where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    let addr = std::net::SocketAddr::new(websocket.host, websocket.port);
    let tcp_listener = TcpListener::bind(&addr).await.expect("bind addr failed");
    let conns = Arc::new(RwLock::new(Vec::new()));
    let move_conns = conns.clone();
    let join = tokio::spawn(async move {
        while let Ok((stream, _)) = tcp_listener.accept().await {
            let join = tokio::spawn(handle_conn(stream, broadcaster.subscribe(), sender.clone()));
            {
                let mut lockconns = move_conns.write().await;
                lockconns.push(join);
            }
        }
    });
    WebSocketServer {
        listner: join,
        conns,
    }
}

async fn handle_conn<E, A, R>(
    stream: TcpStream,
    listener: crate::impls::CustomEventListner<E>,
    sender: crate::impls::CustomActionSender<A, R>,
) where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    let _addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
    super::websocket_loop(ws_stream, listener, sender).await
}
