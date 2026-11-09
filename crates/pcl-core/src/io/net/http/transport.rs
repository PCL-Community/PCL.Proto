//! Shared HTTP clients and protocol selection for API requests and file transfers.
use super::resources::{DownloadResources, HttpMode};
use reqwest::header::HeaderMap;
use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

/// A resolved request destination and the headers required by that destination.
pub struct RequestTarget {
    pub url: String,
    pub headers: HeaderMap,
}

impl RequestTarget {
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: HeaderMap::new(),
        }
    }
}

#[derive(Clone)]
pub struct HttpTransport {
    resources: Arc<DownloadResources>,
    clients: Arc<RwLock<(reqwest::Client, reqwest::Client)>>,
}

impl Default for HttpTransport {
    fn default() -> Self {
        Self::new(default_client(false), default_client(true))
    }
}

impl HttpTransport {
    pub fn new(client: reqwest::Client, http1: reqwest::Client) -> Self {
        Self {
            clients: Arc::new(RwLock::new((client, http1))),
            resources: Arc::new(DownloadResources::default()),
        }
    }

    /// Clones share clients, bandwidth budget and connection quota.
    pub fn resources(&self) -> &Arc<DownloadResources> {
        &self.resources
    }

    pub fn http_client(&self) -> reqwest::Client {
        self.clients.read().unwrap().0.clone()
    }

    /// In-flight requests finish on their original client.
    pub fn replace_clients(&self, client: reqwest::Client, http1: reqwest::Client) {
        *self.clients.write().unwrap() = (client, http1);
    }

    /// Sends a GET; callers hold the quota lease and account for streamed response bytes.
    pub async fn send_request(
        &self,
        mode: HttpMode,
        large: bool,
        target: &RequestTarget,
        range: Option<(u64, u64)>,
        timeout: Option<Duration>,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = &target.url;
        let send = |client: &reqwest::Client| {
            let mut request = client.get(url).headers(target.headers.clone());
            // Range 和大文件保留原始字节；普通请求交给客户端协商压缩。
            if large || range.is_some() {
                request = request.header(reqwest::header::ACCEPT_ENCODING, "identity");
            }
            if let Some((start, end)) = range {
                request = request.header(reqwest::header::RANGE, format!("bytes={start}-{end}"));
            }
            if let Some(timeout) = timeout {
                request = request.timeout(timeout);
            }
            request.send()
        };
        let (multiplexed, http1) = self.clients.read().unwrap().clone();
        let client = match mode {
            HttpMode::Http11 => &http1,
            HttpMode::Auto if large => &http1,
            _ => &multiplexed,
        };
        let can_fallback = !std::ptr::eq(client, &http1);
        match send(client).await {
            Err(error) if can_fallback && !error.is_timeout() => {
                log::debug!("HTTP negotiation failed for {url}, retrying HTTP/1.1: {error}");
                send(&http1).await
            }
            result => result,
        }
    }
}

fn default_client(http1: bool) -> reqwest::Client {
    let mut builder = reqwest::Client::builder()
        .user_agent(concat!("pcl-core/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(15))
        .read_timeout(Duration::from_secs(15))
        .pool_max_idle_per_host(64)
        .pool_idle_timeout(Duration::from_secs(90));
    if http1 {
        builder = builder.http1_only();
    }
    builder
        .build()
        .expect("static HTTP transport configuration")
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, http::HeaderMap, routing::get};
    use std::io::Write;

    #[tokio::test]
    async fn compression_preserves_range_and_large_transfer_bytes() {
        const BODY: &[u8] = b"{\"data\":[{\"id\":1}]}";
        let mut gzip = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::default());
        gzip.write_all(BODY).unwrap();
        let compressed = gzip.finish().unwrap();
        let app = Router::new()
            .route(
                "/metadata",
                get(move |headers: HeaderMap| {
                    let compressed = compressed.clone();
                    async move {
                        assert!(
                            headers["accept-encoding"]
                                .to_str()
                                .unwrap()
                                .contains("gzip")
                        );
                        ([("content-encoding", "gzip")], compressed)
                    }
                }),
            )
            .route(
                "/file",
                get(|headers: HeaderMap| async move {
                    assert_eq!(headers["accept-encoding"], "identity");
                    BODY
                }),
            );
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
        let transport = HttpTransport::default();
        for (mode, large, range, path) in [
            (HttpMode::Auto, false, None, "metadata"),
            (HttpMode::Http11, false, None, "metadata"),
            (HttpMode::Auto, true, None, "file"),
            (HttpMode::Auto, false, Some((0, 0)), "file"),
        ] {
            let response = transport
                .send_request(
                    mode,
                    large,
                    &RequestTarget::new(format!("{base}/{path}")),
                    range,
                    Some(Duration::from_secs(5)),
                )
                .await
                .unwrap();
            assert_eq!(response.bytes().await.unwrap().as_ref(), BODY);
        }
        server.abort();
    }
}
