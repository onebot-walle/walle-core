use std::sync::Arc;
use tokio::{
    net::{TcpListener, TcpStream},
    sync::RwLock,
};

use crate::{app::CustomOneBot, comms::WebSocketServer, config::WebSocketServer as wssc};

pub async fn run<E, A, R>(config: &wssc, ob: Arc<CustomOneBot<E, A, R>>) -> WebSocketServer
where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    let addr = std::net::SocketAddr::new(config.host, config.port);
    let tcp_listener = TcpListener::bind(&addr).await.expect("bind addr failed");
    let conns = Arc::new(RwLock::new(Vec::new()));
    let move_conns = conns.clone();
    let access_token = config.access_token.clone();
    let join = tokio::spawn(async move {
        while let Ok((stream, _)) = tcp_listener.accept().await {
            let join = tokio::spawn(handle_conn(access_token.clone(), stream, ob.clone()));
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
    access_token: Option<String>,
    stream: TcpStream,
    ob: Arc<CustomOneBot<E, A, R>>,
) where
    E: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
    A: Clone + serde::Serialize + Send + 'static + std::fmt::Debug,
    R: Clone + serde::de::DeserializeOwned + Send + 'static + std::fmt::Debug,
{
    if let Some(ws_stream) = crate::comms::util::upgrade_websocket(&access_token, stream).await {
        super::websocket_loop(ws_stream, ob).await
    }
}
