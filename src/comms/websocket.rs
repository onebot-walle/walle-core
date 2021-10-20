use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub async fn run(addr: std::net::SocketAddr) -> (JoinHandle<()>, Arc<RwLock<Vec<JoinHandle<()>>>>) {
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("bind addr failed");
    let conns = Arc::new(RwLock::new(Vec::new()));
    let move_conns = conns.clone();
    let join = tokio::spawn(async move {
        while let Ok((stream, _)) = listener.accept().await {
            let join = tokio::spawn(handle_conn(stream));
            {
                let mut lockconns = move_conns.write().await;
                lockconns.push(join);
            }
        }
    });
    (join, conns)
}

async fn handle_conn(stream: TcpStream) {
    let _addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    let _ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    // todo
}
