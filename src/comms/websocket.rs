use crate::config::WebSocket;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;

pub struct WebSocketServer {
    pub listner: JoinHandle<()>,
    pub conns: Arc<RwLock<Vec<(JoinHandle<()>, JoinHandle<()>)>>>,
}

impl WebSocketServer {
    pub(crate) async fn abort(self) {
        let mut conns = self.conns.write().await;
        for conn in conns.iter_mut() {
            conn.0.abort();
            conn.1.abort();
        }
        self.listner.abort();
    }
}

#[cfg(feature = "impl")]
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
    WebSocketServer {
        listner: join,
        conns,
    }
}

#[cfg(feature = "impl")]
async fn handle_conn<E, A, R>(
    stream: TcpStream,
    listener: crate::impls::CustomEventListner<E>,
    sender: crate::impls::CustomActionSender<A, R>,
) -> (JoinHandle<()>, JoinHandle<()>)
where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: Serialize + std::fmt::Debug + Send + 'static,
{
    let _addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");
    super::util::websocket_loop(ws_stream, listener, sender).await
}

// #[cfg(feature = "sdk")]
// pub async fn sdk_handle_conn(stream: TcpStream) -> (JoinHandle<()>, JoinHandle<()>) {
//     todo!()
// }
