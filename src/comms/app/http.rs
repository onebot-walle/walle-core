use std::sync::Arc;
use std::{fmt::Debug, time::Duration};

use hyper::body::Buf;
use hyper::{
    client::HttpConnector, header::CONTENT_TYPE, Body, Client as HyperClient, Method, Request,
};
use tracing::warn;

use crate::{
    app::{CustomRespSender, OneBot},
    HttpClient, ProtocolItem, SelfId,
};

impl<E, A, R, const V: u8> OneBot<E, A, R, V>
where
    E: ProtocolItem + SelfId + Clone + Send + 'static + Debug,
    A: ProtocolItem + Clone + Send + 'static + Debug,
    R: ProtocolItem + Clone + Send + 'static + Debug,
{
    pub(crate) async fn http(self: &Arc<Self>) {
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
            tokio::spawn(async move {
                while ob.is_running() {
                    while let Some((action, action_tx)) = rx.recv().await {
                        ob.http_push(action, action_tx, &cli, &http).await;
                    }
                }
            });
        }
        self.set_running();
    }

    async fn http_push(
        self: &Arc<Self>,
        action: A,
        action_tx: CustomRespSender<R>,
        cli: &HyperClient<HttpConnector>,
        http: &HttpClient,
    ) {
        use crate::comms::utils::AuthReqHeaderExt;
        let data = action.json_encode();
        let req = Request::builder()
            .method(Method::POST)
            .uri(&http.url)
            .header(CONTENT_TYPE, "application/json")
            .header_auth_token(&http.access_token)
            .body(Body::from(data))
            .unwrap();
        let resp =
            match tokio::time::timeout(Duration::from_secs(http.timeout), cli.request(req)).await {
                Ok(Ok(r)) => r,
                Ok(Err(e)) => {
                    warn!(target: "Walle-core", "HTTP request error: {}", e);
                    return;
                }
                Err(_) => {
                    warn!(target: "Walle-core", "call action timeout");
                    return;
                }
            };
        let body = hyper::body::aggregate(resp).await.unwrap(); //todo
        let resp: R = serde_json::from_reader(body.reader()).unwrap();
        action_tx.send(resp).ok();
    }
}
