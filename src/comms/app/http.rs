use std::sync::Arc;
use std::{fmt::Debug, time::Duration};

use hyper::body::Buf;
use hyper::client::HttpConnector;
use hyper::header::CONTENT_TYPE;
use hyper::{Client as HyperClient, Method, Request};
use tokio::task::JoinHandle;
use tracing::warn;

use crate::action::ActionType;
use crate::config::HttpClient;
use crate::utils::ProtocolItem;
use crate::{
    app::{OneBot, OneshotSender},
    handle::EventHandler,
    SelfId,
};

impl<E, A, R, H, const V: u8> OneBot<E, A, R, H, V>
where
    E: ProtocolItem + Clone + Debug + SelfId,
    A: ProtocolItem + Clone + Debug + ActionType,
    R: ProtocolItem + Clone + Debug,
    H: EventHandler<E, A, R> + Send + Sync + 'static,
{
    pub(crate) async fn http(self: &Arc<Self>, joins: &mut Vec<JoinHandle<()>>) {
        if self.config.http.is_empty() {
            return;
        }
        let client = Arc::new(HyperClient::new());
        for (bot_id, http) in self.config.http.iter() {
            let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
            self.insert_bot(bot_id, &tx).await;
            let ob = self.clone();
            let cli = client.clone();
            let http = http.clone();
            joins.push(tokio::spawn(async move {
                while ob.is_running() {
                    while let Some((action, action_tx)) = rx.recv().await {
                        ob.http_push(action, action_tx, &cli, &http).await;
                    }
                }
            }));
        }
        self.set_running();
    }

    async fn http_push(
        self: &Arc<Self>,
        action: A,
        action_tx: Option<OneshotSender<R>>,
        cli: &HyperClient<HttpConnector>,
        http: &HttpClient,
    ) {
        use crate::comms::utils::AuthReqHeaderExt;
        if let Some(action_tx) = action_tx {
            let content_type = action.content_type();
            let req = Request::builder()
                .method(Method::POST)
                .uri(&http.url)
                .header_auth_token(&http.access_token)
                .header(CONTENT_TYPE, content_type.to_string())
                .body(action.to_body(content_type))
                .unwrap();
            let resp =
                match tokio::time::timeout(Duration::from_secs(http.timeout), cli.request(req))
                    .await
                {
                    Ok(Ok(r)) => r,
                    Ok(Err(e)) => {
                        warn!(target: crate::WALLE_CORE, "HTTP request error: {}", e);
                        return;
                    }
                    Err(_) => {
                        warn!(target: crate::WALLE_CORE, "call action timeout");
                        return;
                    }
                };
            let body = hyper::body::aggregate(resp).await.unwrap(); //todo
            let resp: R = serde_json::from_reader(body.reader()).unwrap();
            action_tx.send(resp).ok();
        }
    }
}
