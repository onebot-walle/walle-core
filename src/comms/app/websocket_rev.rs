use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use crate::{comms::WebSocketServer, config::WebSocket};

pub async fn run<E, A, R>(
    config: &WebSocket,
    event_handler: crate::app::ArcEventHandler<E>,
    action_broadcaster: crate::app::CustomActionBroadcaster<A, R>,
) -> WebSocketServer
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    let addr = std::net::SocketAddr::new(config.host, config.port);
    let tcp_listener = TcpListener::bind(&addr).await.expect("bind addr failed");
    let conns = Arc::new(RwLock::new(Vec::new()));
    let move_conns = conns.clone();
    let join = tokio::spawn(async move {
        while let Ok((stream, _)) = tcp_listener.accept().await {
            let join = tokio::spawn(handle_conn(
                stream,
                event_handler.clone(),
                action_broadcaster.subscribe(),
            ));
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
    event_handler: crate::app::ArcEventHandler<E>,
    action_listener: crate::app::CustomActionListenr<A, R>,
) where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    let _addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    let ws_stream = tokio_tungstenite::accept_async(stream).await.unwrap();
    super::websocket_loop(ws_stream, event_handler, action_listener).await
}
