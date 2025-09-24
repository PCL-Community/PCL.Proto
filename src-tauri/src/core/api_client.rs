use crate::{core::downloader, setup::constants::USER_AGENT};
use reqwest::Client;
use serde::{Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

#[derive(serde::Serialize, serde::Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ApiProvider {
    #[default]
    Official,
    BMCLApi,
}

/// TODO)) 需要在下载时截取后面的部分才能正常使用第三方API
/// 否则还是用的官方API
#[derive(Clone)]
pub struct ApiBases {
    pub meta_base: &'static str,
    pub resources_base: &'static str,
}

impl ApiBases {
    pub fn new(provider: &ApiProvider) -> Self {
        match provider {
            ApiProvider::Official => ApiBases {
                meta_base: "https://piston-meta.mojang.com",
                resources_base: "https://resources.download.minecraft.net",
            },
            ApiProvider::BMCLApi => ApiBases {
                meta_base: "https://bmclapi2.bangbang93.com",
                resources_base: "https://bmclapi2.bangbang93.com/assets",
            },
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum McApiError {
    #[error("HTTP request failed: {0}")]
    RequestError(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("sha1 check failed: {0}")]
    Sha1Mismatch(PathBuf),

    #[error("progress sender: {0}")]
    ProgressSenderError(#[from] tokio::sync::mpsc::error::SendError<downloader::ProgressUpdate>),
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
        pub libraries: Vec<LibraryItem>,
        pub assets: String,
        #[serde(rename = "assetIndex")]
        pub asset_index: DownloadInfo,
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
        pub path: Option<String>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct Rule {
        pub action: String, // allow or not allow
        pub os: OSRule,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct OSRule {
        pub name: crate::core::platform::OS,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LibraryItem {
        pub name: String,
        pub downloads: LibraryDownloads,
        pub rules: Option<Vec<Rule>>,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LibraryDownloads {
        pub artifact: DownloadInfo,
    }

    pub const VERSION_MANIFEST_ENDPOINT: &str = "mc/game/version_manifest.json";
}

pub struct MinecraftApiClient {
    client: Client,
    api_bases: RwLock<ApiBases>,
    cache: RwLock<HashMap<String, (Instant, serde_json::Value)>>,
    ttl: Duration,
}

impl MinecraftApiClient {
    /// Create a new MinecraftApiClient
    pub fn new(client: Client, api_provider: &ApiProvider) -> Self {
        Self {
            client,
            api_bases: RwLock::new(ApiBases::new(api_provider)),
            cache: RwLock::new(HashMap::new()),
            ttl: Duration::from_secs(60 * 5),
        }
    }

    pub fn switch_api_bases(&self, provider: &ApiProvider) {
        let mut guard = self.api_bases.blocking_write();
        *guard = ApiBases::new(provider);
        drop(guard);
    }

    pub fn api_bases(&self) -> ApiBases {
        let guard = self.api_bases.blocking_read();
        guard.clone()
    }

    pub async fn api_bases_async(&self) -> ApiBases {
        let guard = self.api_bases.read().await;
        guard.clone()
    }

    /// Get data from a URL
    async fn get_inner<T: DeserializeOwned>(&self, url: &str) -> Result<T, McApiError> {
        let response = self
            .client
            .get(url)
            .header(reqwest::header::USER_AGENT, USER_AGENT)
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
        let url = format!("{}/{}", self.api_bases.read().await.meta_base, endpoint);
        let data = self.get(&url, allow_from_cache).await?;
        Ok(data)
    }

    /// Get version details from version id and save the temp
    pub async fn get_version_details(
        &self,
        version_id: &str,
        temp_dir: &Path,
    ) -> Result<game::VersionDetails, McApiError> {
        // STEP1: get the version json
        let endpoint = game::VERSION_MANIFEST_ENDPOINT;
        let manifest: game::VersionManifest = self.get_with_endpoint(&endpoint, true).await?;
        let version = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or(McApiError::VersionNotFound(version_id.to_string()))?;
        let version_json = self.get::<serde_json::Value>(&version.url, false).await?;
        // save version detail json to temp_dir
        tokio::fs::create_dir_all(temp_dir).await?;
        let version_json_path = temp_dir.join(format!("{}.json", version_id));
        let version_json_str = serde_json::to_string(&version_json)?;
        tokio::fs::write(&version_json_path, version_json_str).await?;
        let version_detail: game::VersionDetails = serde_json::from_value(version_json)?;
        Ok(version_detail)
    }
}

#[tokio::test]
async fn mc_manifest() {
    let mc_api_client = MinecraftApiClient::new(Client::new(), &ApiProvider::Official);
    let manifest: game::VersionManifest = mc_api_client
        .get_with_endpoint(game::VERSION_MANIFEST_ENDPOINT, true)
        .await
        .unwrap();
    println!("{:?}", manifest);
}
