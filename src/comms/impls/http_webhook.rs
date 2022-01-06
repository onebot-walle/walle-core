use hyper::{body::Buf, client::HttpConnector, Body, Client as HyperClient, Method, Request, Uri};
use serde::{de::DeserializeOwned, Serialize};
use tokio::task::JoinHandle;

use crate::utils::Echo;

#[cfg(feature = "impl")]
impl<E, A, R> crate::impls::CustomOneBot<E, A, R>
where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: std::fmt::Debug + Serialize + Send + 'static,
{
    pub(crate) fn build_webhook_clients(
        &self,
        handler: crate::impls::ArcActionHandler<A, R>,
    ) -> Vec<Client<E, A, R>> {
        let mut r = vec![];
        for webhook in &self.config.http_webhook {
            r.push(Client::new(
                "json".to_owned(),
                self.r#impl.clone(),
                self.r#impl.clone(),
                self.platform.clone(),
                self.self_id.clone(),
                handler.clone(),
                self.broadcaster.subscribe(),
                webhook,
            ));
        }
        r
    }
}

#[cfg(feature = "impl")]
pub struct Client<E, A, R> {
    inner: HyperClient<HttpConnector>,
    uri: Uri,
    content_type: String,
    ua: String,
    r#impl: String,
    platform: String,
    self_id: String,
    access_token: Option<String>,
    time_out: u64,
    handler: crate::impls::ArcActionHandler<A, R>,
    listner: crate::impls::CustomEventListner<E>,
}

#[cfg(feature = "impl")]
impl<'de, E, A, R> Client<E, A, R>
where
    E: Clone + Serialize + Send + 'static,
    A: DeserializeOwned + std::fmt::Debug + Send + 'static,
    R: std::fmt::Debug + Serialize + Send + 'static,
{
    pub fn new(
        content_type: String,
        ua: String,
        r#impl: String,
        platform: String,
        self_id: String,
        handler: crate::impls::ArcActionHandler<A, R>,
        listner: crate::impls::CustomEventListner<E>,
        config: &crate::config::HttpWebhook,
    ) -> Self {
        Client {
            inner: HyperClient::new(),
            uri: config.url.parse().unwrap(),
            content_type,
            ua,
            r#impl,
            platform,
            self_id,
            access_token: config.access_token.clone(),
            time_out: config.timeout,
            handler,
            listner,
        }
    }

    pub fn run(mut self) -> JoinHandle<()> {
        tokio::spawn(async move {
            while let Ok(e) = self.listner.recv().await {
                let actions = self.push(e).await;
                if let Some(actions) = actions {
                    for action in actions {
                        self.handler.handle(action).await;
                    }
                }
            }
        })
    }

    async fn push(&self, event: E) -> Option<Vec<A>> {
        let data = match serde_json::to_string(&event) {
            Ok(s) => s,
            Err(_) => {
                // log serde error here
                return None;
            }
        };
        let req_builder = Request::builder()
            .method(Method::POST)
            .uri(&self.uri)
            .header("Content-Type", &self.content_type)
            .header("User-Agent", &self.ua)
            .header("X-OneBot-Version", "12")
            .header("X-Impl", &self.r#impl)
            .header("X-Platform", &self.platform)
            .header("X-Self-ID", &self.self_id);
        let req_builder = match &self.access_token {
            Some(token) => req_builder.header("Authorization", &format!("Bearer {}", token)),
            None => req_builder,
        };
        let req = req_builder
            .body(Body::from(data))
            .expect("Request build failed");

        let resp = match tokio::time::timeout(
            std::time::Duration::from_secs(self.time_out),
            self.inner.request(req),
        )
        .await
        {
            Ok(r) => r.expect("error"),
            Err(_) => {
                // log timeout
                return None;
            }
        };
        if resp.status() == 204 {
            return None; // push success and no action
        }
        if resp.status() != 200 {
            panic!()
            // todo push fail need handle
        }

        let body = hyper::body::aggregate(resp).await.unwrap();
        let echo_action: Vec<Echo<A>> = match serde_json::from_reader(body.reader()) {
            Ok(e) => e,
            Err(_) => {
                panic!()
                // handle error here
            }
        };
        let mut r = vec![];
        for ea in echo_action {
            let (a, _) = ea.unpack();
            r.push(a);
        }
        Some(r)
    }
}
