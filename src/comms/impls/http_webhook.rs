use std::{sync::Arc, time::Duration};

use hyper::{
    body::Buf,
    client::HttpConnector,
    header::{CONTENT_TYPE, USER_AGENT},
    Body, Client as HyperClient, Method, Request, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};

impl<E, A, R, const V: u8> crate::impls::CustomOneBot<E, A, R, V>
where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: std::fmt::Debug + Serialize + Send + 'static,
{
    pub(crate) async fn webhook(self: &Arc<Self>) {
        if self.config.http_webhook.is_empty() {
            return;
        }
        let ob = self.clone();
        let client = Arc::new(HyperClient::new());
        tokio::spawn(async move {
            let mut rx = ob.broadcaster.subscribe();
            while ob.is_running() {
                while let Ok(e) = rx.recv().await {
                    ob.webhook_push(e, &client).await;
                }
            }
        });
        self.set_running();
    }

    async fn webhook_push(self: &Arc<Self>, event: E, client: &Arc<HyperClient<HttpConnector>>) {
        use crate::comms::utils::AuthReqHeaderExt;
        let data = match serde_json::to_string(&event) {
            Ok(s) => s,
            Err(e) => {
                tracing::warn!("{}", e);
                return;
            }
        };
        for webhook in &self.config.http_webhook {
            let req = Request::builder()
                .method(Method::POST)
                .uri(&webhook.url)
                .header(CONTENT_TYPE, "application/json")
                .header(
                    USER_AGENT,
                    format!("OneBot/{} ({}) Walle/{}", V, self.platform, crate::VERSION),
                )
                .header("X-OneBot-Version", V.to_string())
                .header("X-Impl", &self.r#impl)
                .header("X-Platform", &self.platform)
                .header("X-Self-ID", self.self_id.read().await.as_str())
                .header_auth_token(&webhook.access_token)
                .body(Body::from(data.clone()))
                .unwrap();
            let ob = self.clone();
            let client = client.clone();
            let timeout = webhook.timeout;
            tokio::spawn(async move {
                let resp =
                    match tokio::time::timeout(Duration::from_secs(timeout), client.request(req))
                        .await
                    {
                        Ok(Ok(r)) => r,
                        Ok(Err(e)) => {
                            tracing::warn!("{}", e);
                            return;
                        }
                        Err(_) => {
                            tracing::warn!("push timeout");
                            return;
                        }
                    };
                match resp.status() {
                    StatusCode::NO_CONTENT => (),
                    StatusCode::OK => {
                        let body = hyper::body::aggregate(resp).await.unwrap();
                        let actions: Vec<A> = match serde_json::from_reader(body.reader()) {
                            Ok(e) => e,
                            Err(_) => {
                                panic!()
                                // handle error here
                            }
                        };
                        for a in actions {
                            let _ = ob.action_handler.handle(a, &ob).await;
                        }
                    }
                    x => tracing::info!("unhandle webhook push status: {}", x),
                }
            });
        }
    }
}
