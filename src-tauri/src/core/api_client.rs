use crate::{
    core::{api_client::plugins::McPluginReport, downloader},
    setup::{ConfigManager, constants::USER_AGENT},
};
use reqwest::Client;
use serde::{
    Serialize,
    de::{DeserializeOwned, Error},
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};
use tokio::sync::RwLock;

#[derive(serde::Serialize, serde::Deserialize, Default, Clone)]
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
    pub forge_base: &'static str,
    pub fabric_base: &'static str,
}

impl ApiBases {
    pub fn new(provider: &ApiProvider) -> Self {
        match provider {
            ApiProvider::Official => ApiBases {
                meta_base: "https://piston-meta.mojang.com",
                resources_base: "https://resources.download.minecraft.net",
                forge_base: "https://maven.minecraftforge.net/net/minecraftforge/forge",
                fabric_base: "https://meta.fabricmc.net/v2/versions/loader",
            },
            ApiProvider::BMCLApi => ApiBases {
                meta_base: "https://bmclapi2.bangbang93.com",
                resources_base: "https://bmclapi2.bangbang93.com/assets",
                forge_base: "https://bmclapi2.bangbang93.com/forge",
                fabric_base: "https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader",
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

    #[error("xml parse error: {0}")]
    XmlError(#[from] quick_xml::DeError),

    #[error("plugin type {0:?} not supported")]
    PluginMismatch(super::mcmod::PluginType),
}

/// Result type for Minecraft API operations.
pub type McApiResult<T> = Result<T, McApiError>;

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
    #[serde(rename_all = "camelCase")]
    pub struct VersionInfo {
        pub id: String,
        pub r#type: String,
        pub url: String,
        pub time: String,
        pub release_time: String,
    }

    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct VersionDetails {
        pub id: String,
        pub r#type: String,
        pub downloads: VersionDownloads,
        pub libraries: Vec<LibraryItem>,
        pub assets: String,
        pub asset_index: DownloadInfo,
        pub main_class: String,
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

    impl Rule {
        pub fn allow(&self) -> bool {
            return self.action == "allow" && self.os.name.is_current();
        }
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

    impl LibraryItem {
        pub fn rule_allow(&self) -> bool {
            if let Some(rules) = &self.rules {
                return rules.iter().any(|rule| rule.allow());
            }
            return true;
        }
    }
}

pub mod plugins {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct McPluginReport {
        pub version: String,
        pub stable: Option<bool>,
    }

    pub mod forge_xml {
        use crate::core::api_client::plugins::McPluginReport;

        #[derive(serde::Deserialize)]
        pub struct Metadata {
            versioning: Versioning,
        }

        #[derive(serde::Deserialize)]
        struct Versioning {
            versions: Versions,
        }

        #[derive(Debug, serde::Deserialize)]
        struct Versions {
            version: Vec<String>,
        }

        impl Metadata {
            pub fn find_versions_of_game(&self, mc_version: &str) -> Vec<McPluginReport> {
                let mut versions = Vec::new();
                for version in &self.versioning.versions.version {
                    let version_split = version.split_once('-');
                    if let Some((mc_version_i, forge_version)) = version_split
                        && mc_version_i == mc_version
                    {
                        versions.push(McPluginReport {
                            version: forge_version.to_string(),
                            stable: None,
                        });
                    }
                }
                versions
            }
        }
    }

    pub mod forge_bmcl {
        #[derive(serde::Deserialize, serde::Serialize)]
        pub struct ForgeVersion {
            mcversion: String,
            modified: String,
            pub version: String,
            files: Vec<File>,
        }

        #[derive(serde::Deserialize, serde::Serialize)]
        struct File {
            format: String,
            category: String,
            hash: String,
        }
    }
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

    /// the provider should be managed in setup info, this will be called once the setup changed
    pub fn switch_provider(&self, provider: &ApiProvider) {
        let mut guard = self.api_bases.blocking_write();
        *guard = ApiBases::new(provider);
        drop(guard);
    }

    pub async fn api_bases_async(&self) -> ApiBases {
        let guard = self.api_bases.read().await;
        guard.clone()
    }

    /// Get data from a URL
    async fn get_inner<T: DeserializeOwned>(&self, url: &str) -> McApiResult<T> {
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
    ) -> McApiResult<T> {
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
    ) -> McApiResult<T> {
        let url = format!("{}/{}", self.api_bases.read().await.meta_base, endpoint);
        let data = self.get(&url, allow_from_cache).await?;
        Ok(data)
    }

    /// Get version details from version id and save the temp
    pub async fn get_version_details(
        &self,
        version_id: &str,
        temp_dir: &Path,
    ) -> McApiResult<game::VersionDetails> {
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

    /// Get forge version details
    pub async fn get_forge_versions(&self, version_id: &str) -> McApiResult<Vec<McPluginReport>> {
        let guard = ConfigManager::instance().app_state.lock().await;
        let current_provider = &guard.pcl_setup_info.api_provider;
        let forge_base = self.api_bases.read().await.forge_base;
        match *current_provider {
            ApiProvider::Official => {
                drop(guard);
                let url = format!("{forge_base}/maven-metadata.xml");
                let response = self
                    .client
                    .get(url)
                    .header(reqwest::header::USER_AGENT, USER_AGENT)
                    .send()
                    .await?;
                let xml_text = response.text().await?;
                let metadata: plugins::forge_xml::Metadata = quick_xml::de::from_str(&xml_text)?;
                let matched_versions = metadata.find_versions_of_game(version_id);
                return Ok(matched_versions);
            }
            ApiProvider::BMCLApi => {
                drop(guard);
                let url_index = format!("{forge_base}/minecraft");
                let indexes: Vec<String> = self.get(&url_index, true).await?;
                log::debug!("BMCL indexes count: {:?}", indexes.len());
                if indexes.iter().all(|index| index != version_id) {
                    return Err(McApiError::VersionNotFound(version_id.to_string()));
                }
                let forge_info: Vec<plugins::forge_bmcl::ForgeVersion> = self
                    .get(&format!("{forge_base}/minecraft/{version_id}"), true)
                    .await?;
                log::debug!("BMCL forge info count: {:?}", forge_info.len());
                return Ok(forge_info
                    .iter()
                    .map(|info| McPluginReport {
                        version: info.version.clone(),
                        stable: None,
                    })
                    .collect());
            }
        }
    }

    pub async fn get_fabric_versions(&self, version_id: &str) -> McApiResult<Vec<McPluginReport>> {
        let fabric_base = self.api_bases.read().await.fabric_base;
        let meta_url = format!("{fabric_base}/{version_id}");
        let fabric_versions: serde_json::Value = self.get(&meta_url, true).await?;
        let loaders = fabric_versions
            .as_array()
            .ok_or_else(|| serde_json::Error::custom("fabric list failed to parse"))?
            .iter()
            .filter_map(|v| {
                let v = v.as_object()?;
                let loader = v.get("loader")?.as_object()?;
                let version = loader.get("version")?.as_str()?;
                let stable = loader.get("stable")?.as_bool();
                Some(McPluginReport {
                    version: version.to_string(),
                    stable: stable,
                })
            })
            .collect();
        Ok(loaders)
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

#[test]
fn forge_test() {
    let mc_api_client = &ConfigManager::instance().api_client;
    let rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let re = mc_api_client.get_forge_versions("1.21.9").await;
        assert!(re.is_ok());
        println!("{re:?}");
    });
}
