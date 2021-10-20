use hyper::{client::HttpConnector, Body, Client as HyperClient, Method, Request, Response, Uri};

pub struct Client {
    inner: HyperClient<HttpConnector>,
    uri: Uri,
    content_type: String,
    ua: String,
    r#impl: String,
    platform: String,
    self_id: String,
    access_token: Option<String>,
}

impl Client {
    pub fn new(
        uri: String,
        content_type: String,
        ua: String,
        r#impl: String,
        platform: String,
        self_id: String,
        access_token: Option<String>,
    ) -> Self {
        Client {
            inner: HyperClient::new(),
            uri: uri.parse().unwrap(),
            content_type,
            ua,
            r#impl,
            platform,
            self_id,
            access_token,
        }
    }

    pub async fn post<T>(&self, data: T) -> Response<Body>
    where
        Body: From<T>,
    {
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

        self.inner.request(req).await.expect("Request error")
    }
}
