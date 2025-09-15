use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use crate::setup::constants::USER_AGENT;
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;
use tokio::sync::RwLock;

#[derive(serde::Serialize, serde::Deserialize)]
pub enum ApiSource {
    Official,
    BMCLApi,
}

#[derive(Error, Debug)]
pub enum McApiError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("Download failed: {0}")]
    DownloadFailed(String),

    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
}

pub mod game {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VersionManifest {
        pub latest: LatestVersions,
        pub versions: Vec<VersionInfo>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LatestVersions {
        pub release: String,
        pub snapshot: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VersionInfo {
        pub id: String,
        #[serde(rename = "type")]
        pub version_type: String,
        pub url: String,
        pub time: String,
        #[serde(rename = "releaseTime")]
        pub release_time: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VersionDetails {
        pub id: String,
        #[serde(rename = "type")]
        pub version_type: String,
        pub downloads: VersionDownloads,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct VersionDownloads {
        pub client: DownloadInfo,
        pub server: DownloadInfo,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct DownloadInfo {
        pub sha1: String,
        pub size: u64,
        pub url: String,
    }

    pub const VERSION_MANIFEST_ENDPOINT: &str = "mc/game/version_manifest.json";
}

#[derive(Clone)]
pub struct MinecraftApiClient {
    client: Arc<Client>,
    base_url: String,
    cache: Arc<RwLock<HashMap<String, (Instant, serde_json::Value)>>>,
    ttl: Duration,
}

impl MinecraftApiClient {
    /// Create a new MinecraftApiClient
    pub fn new(client: Arc<Client>, base_url: impl Into<String>) -> Self {
        Self {
            client,
            base_url: base_url.into(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            ttl: Duration::from_secs(60 * 5),
        }
    }

    /// Get data from a URL
    async fn get_inner<T: DeserializeOwned>(&self, url: &str) -> Result<T, McApiError> {
        let response = self
            .client
            .get(url)
            .header("User-Agent", USER_AGENT)
            .send()
            .await?;
        let data: T = response.json().await?;
        log::info!("got data from {}", url);
        Ok(data)
    }

    /// Get data from a URL with cache
    pub async fn get<T: DeserializeOwned + Serialize>(
        &self,
        url: &str,
        allow_from_cache: bool,
    ) -> Result<T, McApiError> {
        let now = Instant::now();
        if allow_from_cache {
            let cache = self.cache.read().await;
            if let Some((cached_at, value)) = cache.get(url) {
                if now.duration_since(*cached_at) < self.ttl {
                    log::info!("cache hit for {}", url);
                    return Ok(serde_json::from_value(value.clone())?);
                }
            }
        }
        let data = self.get_inner(url).await?;
        let value = serde_json::to_value(&data)?;
        let mut cache = self.cache.write().await;
        cache.insert(url.to_string(), (now, value));
        Ok(data)
    }

    /// Get data from a endpoint with embedded base_url
    pub async fn get_with_endpoint<T: DeserializeOwned + Serialize>(
        &self,
        endpoint: &str,
        allow_from_cache: bool,
    ) -> Result<T, McApiError> {
        let url = format!("{}/{}", self.base_url, endpoint);
        let data = self.get(&url, allow_from_cache).await?;
        Ok(data)
    }
}

#[tokio::test]
async fn mc_manifest() {
    use crate::core::downloader::DOWNLOADER;
    let mc_api_client =
        MinecraftApiClient::new(DOWNLOADER.client.clone(), "https://launchermeta.mojang.com");
    let manifest: game::VersionManifest = mc_api_client
        .get_with_endpoint(game::VERSION_MANIFEST_ENDPOINT, true)
        .await
        .unwrap();
    println!("{:?}", manifest);
}
