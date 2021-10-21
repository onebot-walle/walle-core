use crate::config::WebSocket;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub async fn run(
    websocket: &WebSocket,
    broadcaster: crate::EventBroadcaster,
    sender: crate::ActionSender,
) -> (
    JoinHandle<()>,
    Arc<RwLock<Vec<(JoinHandle<()>, JoinHandle<()>)>>>,
) {
    let addr = std::net::SocketAddr::new(websocket.host, websocket.port);
    let try_socket = TcpListener::bind(&addr).await;
    let tcp_listener = try_socket.expect("bind addr failed");
    let conns = Arc::new(RwLock::new(Vec::new()));
    let move_conns = conns.clone();
    let join = tokio::spawn(async move {
        while let Ok((stream, _)) = tcp_listener.accept().await {
            let join = handle_conn(stream, broadcaster.subscribe(), sender.clone()).await;
            {
                let mut lockconns = move_conns.write().await;
                lockconns.push(join);
            }
        }
    });
    (join, conns)
}

async fn handle_conn(
    stream: TcpStream,
    listener: crate::EventListner,
    sender: crate::ActionSender,
) -> (JoinHandle<()>, JoinHandle<()>) {
    let _addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    super::util::web_socket_loop(ws_stream, listener, sender).await
}
