use crate::{
    app::config::{ConfigManager, constants::USER_AGENT},
    io::net::http::api_client::plugins::McPluginReport,
};
use dashmap::DashMap;
use reqwest::Client;
use serde::{
    Serialize,
    de::{DeserializeOwned, Error},
};
use std::{
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

#[derive(Clone)]
pub struct ApiBases {
    pub meta_base: &'static str,
    pub resources_base: &'static str,
    pub forge_base: &'static str,
    pub fabric_base: &'static str,
    pub legacy_fabric_base: &'static str,
    pub modrinth_base: &'static str,
    pub curseforge_base: &'static str,
}

impl ApiBases {
    pub fn new(provider: &ApiProvider) -> Self {
        match provider {
            ApiProvider::Official => ApiBases {
                meta_base: "https://piston-meta.mojang.com",
                resources_base: "https://resources.download.minecraft.net",
                forge_base: "https://maven.minecraftforge.net/net/minecraftforge/forge",
                fabric_base: "https://meta.fabricmc.net/v2/versions/loader",
                legacy_fabric_base: "https://meta.legacyfabric.net/v2/versions/loader",
                modrinth_base: "https://api.modrinth.com/v2",
                curseforge_base: "https://api.curseforge.com/v1",
            },
            ApiProvider::BMCLApi => ApiBases {
                meta_base: "https://bmclapi2.bangbang93.com",
                resources_base: "https://bmclapi2.bangbang93.com/assets",
                forge_base: "https://bmclapi2.bangbang93.com/forge",
                fabric_base: "https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader",
                legacy_fabric_base: "https://meta.legacyfabric.net/v2/versions/loader",
                modrinth_base: "https://api.modrinth.com/v2",
                curseforge_base: "https://api.curseforge.com/v1",
            },
        }
    }
    /// Mirror only Mojang-owned download hosts; third-party URLs retain their origin.
    pub fn download_url(&self, url: &str) -> String {
        if self.meta_base == "https://piston-meta.mojang.com" {
            return url.into();
        }
        let Ok(parsed) = reqwest::Url::parse(url) else {
            return url.into();
        };
        let prefix = match parsed.host_str() {
            Some(
                "piston-meta.mojang.com"
                | "launchermeta.mojang.com"
                | "piston-data.mojang.com"
                | "launcher.mojang.com",
            ) => self.meta_base.to_owned(),
            Some("libraries.minecraft.net") => format!("{}/maven", self.meta_base),
            Some("resources.download.minecraft.net") => self.resources_base.to_owned(),
            _ => return url.into(),
        };
        let mut rewritten = format!("{prefix}{}", parsed.path());
        if let Some(query) = parsed.query() {
            rewritten.push('?');
            rewritten.push_str(query);
        }
        rewritten
    }
    pub fn download_urls(
        &self,
        url: &str,
        preference: crate::io::download::resources::SourcePreference,
        official_fast: bool,
    ) -> Vec<String> {
        use crate::io::download::resources::SourcePreference;
        // Test/private API bases remain isolated from public network fallbacks.
        if !matches!(
            self.meta_base,
            "https://piston-meta.mojang.com" | "https://bmclapi2.bangbang93.com"
        ) {
            return vec![self.download_url(url)];
        }
        // Fabric metadata has its own source fallback and is not Mojang metadata.
        if url.starts_with("https://bmclapi2.bangbang93.com/fabric-meta/") {
            return vec![url.into()];
        }
        let canonical = if let Some(path) = url.strip_prefix("https://bmclapi2.bangbang93.com/") {
            if let Some(tail) = path.strip_prefix("assets/") {
                format!("https://resources.download.minecraft.net/{tail}")
            } else if let Some(tail) = path
                .strip_prefix("maven/")
                .or_else(|| path.strip_prefix("libraries/"))
            {
                format!("https://libraries.minecraft.net/{tail}")
            } else if path.starts_with("mc/game/") || path.ends_with(".json") {
                format!("https://piston-meta.mojang.com/{path}")
            } else if path.starts_with("v1/objects/") {
                format!("https://piston-data.mojang.com/{path}")
            } else {
                return vec![url.into()];
            }
        } else {
            url.to_owned()
        };
        let mirror = Self::new(&ApiProvider::BMCLApi).download_url(&canonical);
        if mirror == canonical {
            return vec![canonical];
        }
        let mut mirrors = vec![mirror.clone()];
        if let Some(tail) = mirror.strip_prefix("https://bmclapi2.bangbang93.com/maven/") {
            mirrors.push(format!("https://bmclapi2.bangbang93.com/libraries/{tail}"));
        }
        if matches!(preference, SourcePreference::Official)
            || (matches!(preference, SourcePreference::Auto) && official_fast)
        {
            let mut urls = vec![canonical];
            urls.extend(mirrors);
            urls
        } else {
            mirrors.push(canonical);
            mirrors
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

    #[error("Invalid Minecraft metadata: {0}")]
    InvalidMetadata(String),

    #[error("API key for {0} is not configured")]
    MissingApiKey(&'static str),

    #[error("Download source does not support reliable HTTP Range")]
    RangeUnsupported,

    #[error("Download segment is significantly slower than its peers")]
    SlowSegment,

    #[error("Version not found: {0}")]
    VersionNotFound(String),

    #[error("sha1 check failed: {0}")]
    Sha1Mismatch(PathBuf),

    #[error("xml parse error: {0}")]
    XmlError(#[from] quick_xml::DeError),

    #[error("plugin type {0:?} not supported")]
    PluginMismatch(crate::minecraft::mod_loader::ModLoaderType),
}

/// Result type for Minecraft API operations.
pub type McApiResult<T> = Result<T, McApiError>;

#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FabricFamily {
    Fabric,
    Legacyfabric,
}

pub mod game {
    pub use crate::minecraft::version::*;
}

pub mod plugins {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct McPluginReport {
        pub version: String,
        pub stable: Option<bool>,
        pub release_time: Option<String>,
    }

    pub mod forge_xml {
        use crate::io::net::http::api_client::plugins::McPluginReport;

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
                            release_time: None,
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
            pub modified: String,
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
    client: std::sync::RwLock<Client>,
    api_bases: RwLock<ApiBases>,
    cache: DashMap<String, (Instant, serde_json::Value)>,
    ttl: Duration,
}

impl MinecraftApiClient {
    /// Create a new MinecraftApiClient
    pub fn new(client: Client, api_provider: &ApiProvider) -> Self {
        Self {
            client: std::sync::RwLock::new(client),
            api_bases: RwLock::new(ApiBases::new(api_provider)),
            cache: DashMap::new(),
            ttl: Duration::from_secs(60 * 5),
        }
    }

    /// Rebuild the dedicated client after the proxy policy changed.
    pub fn rebuild_client(&self) -> Result<(), String> {
        let client = crate::io::net::proxy::builder()
            .build()
            .map_err(|error| error.to_string())?;
        *self.client.write().unwrap() = client;
        Ok(())
    }

    #[cfg(test)]
    pub(crate) fn with_test_bases(bases: ApiBases) -> Self {
        let mut client = Self::new(Client::new(), &ApiProvider::Official);
        client.api_bases = RwLock::new(bases);
        client
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
        self.fetch_checked(url, |bytes| Ok(serde_json::from_slice(bytes)?))
            .await
    }

    async fn fetch_checked<T>(
        &self,
        url: &str,
        validate: impl Fn(&[u8]) -> McApiResult<T>,
    ) -> McApiResult<T> {
        use crate::io::download::resources::{DOWNLOAD_RESOURCES, SourcePreference};
        use futures_util::StreamExt;
        use std::sync::atomic::Ordering;
        let settings = DOWNLOAD_RESOURCES.settings();
        let manifest = url.ends_with(game::VERSION_MANIFEST_ENDPOINT);
        let preference = if manifest {
            settings.version_source
        } else {
            settings.file_source
        };
        let urls = self.api_bases.read().await.download_urls(
            url,
            preference,
            manifest || DOWNLOAD_RESOURCES.official_fast.load(Ordering::Relaxed),
        );
        let mut last_error = None;
        for actual_url in urls {
            let official_manifest =
                manifest && actual_url.starts_with("https://piston-meta.mojang.com/");
            let result = async {
                let _lease = DOWNLOAD_RESOURCES.acquire().await;
                let started = Instant::now();
                let timeout = Duration::from_secs(
                    if official_manifest && matches!(preference, SourcePreference::Auto) {
                        5
                    } else {
                        15
                    },
                );
                let response = crate::io::download::send_request(
                    settings.http_mode,
                    false,
                    &actual_url,
                    None,
                    Some(timeout),
                )
                .await?
                .error_for_status()?;
                let mut stream = response.bytes_stream();
                let mut bytes = Vec::new();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    DOWNLOAD_RESOURCES.record_bytes(chunk.len());
                    DOWNLOAD_RESOURCES.throttle(chunk.len()).await;
                    bytes.extend_from_slice(&chunk);
                }
                let value = validate(&bytes)?;
                if official_manifest {
                    DOWNLOAD_RESOURCES.official_fast.store(
                        started.elapsed() < Duration::from_secs(4),
                        Ordering::Relaxed,
                    );
                }
                Ok(value)
            }
            .await;
            if official_manifest && result.is_err() {
                DOWNLOAD_RESOURCES
                    .official_fast
                    .store(false, Ordering::Relaxed);
            }
            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    log::warn!("Metadata source failed for {actual_url}: {error}");
                    last_error = Some(error);
                }
            }
        }
        Err(last_error
            .unwrap_or_else(|| McApiError::InvalidMetadata("No metadata source available".into())))
    }

    /// Get data from a URL with cache
    pub async fn get<T: DeserializeOwned + Serialize>(
        &self,
        url: &str,
        allow_from_cache: bool,
    ) -> McApiResult<T> {
        let now = Instant::now();
        if allow_from_cache {
            if let Some(data) = self.cache_hit(url) {
                return Ok(data);
            }
        }
        let data = self.get_inner(url).await?;
        self.cache_store(url, now, &data);
        Ok(data)
    }

    /// 取回仍在 TTL 内的缓存，并反序列化为目标类型。
    fn cache_hit<T: DeserializeOwned>(&self, url: &str) -> Option<T> {
        let entry = self.cache.get(url)?;
        if Instant::now().duration_since(entry.value().0) >= self.ttl {
            return None;
        }
        let data = serde_json::from_value(entry.value().1.clone()).ok()?;
        log::info!("cache hit for {}", url);
        Some(data)
    }

    fn cache_store<T: Serialize>(&self, url: &str, at: Instant, data: &T) {
        if let Ok(value) = serde_json::to_value(data) {
            self.cache.insert(url.to_string(), (at, value));
        }
    }

    /// Get data from a endpoint with embedded base_url
    pub async fn get_with_endpoint<T: DeserializeOwned + Serialize>(
        &self,
        endpoint: &str,
        allow_from_cache: bool,
    ) -> McApiResult<T> {
        let url = format!("https://piston-meta.mojang.com/{endpoint}");
        self.get(&url, allow_from_cache).await
    }

    /// Get version details from version id and save the temp
    pub async fn get_version_details(
        &self,
        version_id: &str,
        temp_dir: &Path,
    ) -> McApiResult<game::McVersion> {
        crate::io::files::validate_file_name(version_id)
            .map_err(|error| McApiError::InvalidMetadata(error.to_string()))?;
        let version_json = self.get_version_metadata(version_id).await?;
        let details = serde_json::from_value(version_json.clone())?;
        tokio::fs::create_dir_all(temp_dir).await?;
        tokio::fs::write(
            temp_dir.join(format!("{version_id}.json")),
            serde_json::to_vec_pretty(&version_json)?,
        )
        .await?;
        Ok(details)
    }

    /// Preserve every upstream field, including launch arguments and Java requirements.
    pub async fn get_version_metadata(&self, version_id: &str) -> McApiResult<serde_json::Value> {
        use sha1::Digest;
        let manifest: game::McVersionManifest = self
            .get_with_endpoint(game::VERSION_MANIFEST_ENDPOINT, true)
            .await?;
        let version = manifest
            .versions
            .iter()
            .find(|v| v.id == version_id)
            .ok_or_else(|| McApiError::VersionNotFound(version_id.into()))?;
        self.fetch_checked(&version.url, |bytes| {
            if let Some(expected) = &version.sha1 {
                let actual = crate::io::files::sha1_hex(sha1::Sha1::digest(bytes));
                if !actual.eq_ignore_ascii_case(expected) {
                    return Err(McApiError::InvalidMetadata(
                        "Version JSON SHA-1 mismatch".into(),
                    ));
                }
            }
            let value: serde_json::Value = serde_json::from_slice(bytes)?;
            if value["id"].as_str() != Some(version_id) {
                return Err(McApiError::InvalidMetadata(
                    "Version metadata ID mismatch".into(),
                ));
            }
            Ok(value)
        })
        .await
    }

    /// Get forge version details
    pub async fn get_forge_versions(&self, version_id: &str) -> McApiResult<Vec<McPluginReport>> {
        let guard = ConfigManager::instance().app_state.lock().await;
        let current_provider = &guard.launcher_setup.api_provider;
        let forge_base = self.api_bases.read().await.forge_base;
        match *current_provider {
            ApiProvider::Official => {
                drop(guard);
                let url = format!("{forge_base}/maven-metadata.xml");
                let client = self.client.read().unwrap().clone();
                let response = client
                    .get(url)
                    .header(reqwest::header::USER_AGENT, USER_AGENT)
                    .send()
                    .await?
                    .error_for_status()?;
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
                        release_time: Some(info.modified.clone()),
                    })
                    .collect());
            }
        }
    }

    pub async fn get_fabric_versions(
        &self,
        version_id: &str,
        family: FabricFamily,
    ) -> McApiResult<Vec<McPluginReport>> {
        crate::io::files::validate_file_name(version_id)
            .map_err(|error| McApiError::InvalidMetadata(error.to_string()))?;
        let bases = self.api_bases.read().await;
        let base = match family {
            FabricFamily::Fabric => bases.fabric_base,
            FabricFamily::Legacyfabric => bases.legacy_fabric_base,
        };
        drop(bases);
        let official = "https://meta.fabricmc.net/v2/versions/loader";
        let mirror = "https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader";
        let sources = if base == official || base == mirror {
            if matches!(
                crate::io::download::resources::DOWNLOAD_RESOURCES
                    .settings()
                    .version_source,
                crate::io::download::resources::SourcePreference::Official
            ) {
                vec![official, mirror]
            } else {
                vec![mirror, official]
            }
        } else {
            vec![base]
        };
        let mut last_error = None;
        let mut fabric_versions = None;
        for source in sources {
            let mut url = reqwest::Url::parse(source)
                .map_err(|error| McApiError::InvalidMetadata(error.to_string()))?;
            url.path_segments_mut()
                .map_err(|_| McApiError::InvalidMetadata("Invalid Fabric metadata URL".into()))?
                .pop_if_empty()
                .push(version_id);
            match self.get::<serde_json::Value>(url.as_str(), true).await {
                Ok(value) if value.is_array() => {
                    fabric_versions = Some(value);
                    break;
                }
                Ok(_) => {
                    last_error = Some(McApiError::InvalidMetadata(
                        "Invalid Fabric version list".into(),
                    ))
                }
                Err(error) => last_error = Some(error),
            }
        }
        let fabric_versions = fabric_versions.ok_or_else(|| {
            last_error.unwrap_or_else(|| {
                McApiError::InvalidMetadata("No Fabric metadata source available".into())
            })
        })?;
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
                    stable,
                    release_time: None,
                })
            })
            .collect();
        Ok(loaders)
    }

    pub async fn get_fabric_profile(
        &self,
        version_id: &str,
        loader: &str,
        family: FabricFamily,
    ) -> McApiResult<serde_json::Value> {
        use crate::io::download::resources::{DOWNLOAD_RESOURCES, SourcePreference};
        for value in [version_id, loader] {
            crate::io::files::validate_file_name(value)
                .map_err(|error| McApiError::InvalidMetadata(error.to_string()))?;
        }
        let bases = self.api_bases.read().await;
        let configured = match family {
            FabricFamily::Fabric => bases.fabric_base,
            FabricFamily::Legacyfabric => bases.legacy_fabric_base,
        };
        drop(bases);
        let official = "https://meta.fabricmc.net/v2/versions/loader";
        let mirror = "https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader";
        let sources = if configured == official || configured == mirror {
            match DOWNLOAD_RESOURCES.settings().file_source {
                SourcePreference::Official => vec![official, mirror],
                _ => vec![mirror, official],
            }
        } else {
            vec![configured]
        };
        let mut last_error = None;
        for base in sources {
            let mut url = reqwest::Url::parse(base)
                .map_err(|error| McApiError::InvalidMetadata(error.to_string()))?;
            url.path_segments_mut()
                .map_err(|_| McApiError::InvalidMetadata("Invalid Fabric metadata URL".into()))?
                .pop_if_empty()
                .extend([version_id, loader, "profile", "json"]);
            match self
                .fetch_checked(url.as_str(), |bytes| {
                    let profile: serde_json::Value = serde_json::from_slice(bytes)?;
                    let expected = format!("net.fabricmc:fabric-loader:{loader}");
                    if profile["inheritsFrom"].as_str() != Some(version_id)
                        || profile["mainClass"].as_str().is_none_or(str::is_empty)
                        || !profile["libraries"].as_array().is_some_and(|libraries| {
                            libraries
                                .iter()
                                .any(|lib| lib["name"].as_str() == Some(expected.as_str()))
                        })
                    {
                        return Err(McApiError::InvalidMetadata("Invalid Fabric profile".into()));
                    }
                    if matches!(family, FabricFamily::Legacyfabric)
                        && !profile["libraries"].as_array().is_some_and(|libraries| {
                            libraries.iter().any(|lib| {
                                lib["name"].as_str()
                                    == Some(
                                        format!("net.legacyfabric:intermediary:{version_id}")
                                            .as_str(),
                                    )
                            })
                        })
                    {
                        return Err(McApiError::InvalidMetadata(
                            "Invalid Legacy Fabric intermediary".into(),
                        ));
                    }
                    Ok(profile)
                })
                .await
            {
                Ok(profile) => return Ok(profile),
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            McApiError::InvalidMetadata("No Fabric metadata source available".into())
        }))
    }

    pub async fn fetch_with_modrinth<T: DeserializeOwned + Serialize>(
        &self,
        endpoint: &str,
    ) -> McApiResult<T> {
        let modrinth_base = self.api_bases.read().await.modrinth_base;
        let url = format!("{modrinth_base}/{endpoint}");
        self.get(&url, true).await
    }

    /// CurseForge 要求 `x-api-key` 请求头，而 `send_request` 不携带自定义头，
    /// 因此走代理策略一致的专用客户端；缓存与 Modrinth 保持一致。
    pub async fn fetch_with_curseforge<T: DeserializeOwned + Serialize>(
        &self,
        endpoint: &str,
    ) -> McApiResult<T> {
        let api_key = crate::app::secrets::curseforge_api_key()
            .ok_or(McApiError::MissingApiKey("CurseForge"))?;
        let curseforge_base = self.api_bases.read().await.curseforge_base;
        let url = format!("{curseforge_base}/{endpoint}");
        let now = Instant::now();
        if let Some(data) = self.cache_hit(&url) {
            return Ok(data);
        }
        let client = self.client.read().unwrap().clone();
        let data: T = client
            .get(&url)
            .header("x-api-key", api_key)
            .timeout(Duration::from_secs(15))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        self.cache_store(&url, now, &data);
        Ok(data)
    }

    pub async fn get_bytes(&self, url: &str) -> McApiResult<Vec<u8>> {
        self.fetch_checked(url, |bytes| Ok(bytes.to_vec())).await
    }
}

#[tauri::command]
pub async fn fetch_with_modrinth(
    api_client: tauri::State<'_, &MinecraftApiClient>,
    endpoint: &str,
) -> Result<serde_json::Value, String> {
    api_client
        .fetch_with_modrinth(endpoint)
        .await
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn fetch_with_curseforge(
    api_client: tauri::State<'_, &MinecraftApiClient>,
    endpoint: &str,
) -> Result<serde_json::Value, String> {
    api_client
        .fetch_with_curseforge(endpoint)
        .await
        .map_err(|err| err.to_string())
}

/// 未配置密钥时界面需要隐藏 CurseForge 选项（对照 CE 的 SupportCurseForge）。
#[tauri::command]
pub fn curseforge_available() -> bool {
    crate::app::secrets::curseforge_available()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn source_preferences_keep_official_fallback_and_both_library_mirrors() {
        use crate::io::download::resources::SourcePreference;
        let bases = ApiBases::new(&ApiProvider::Official);
        let official = "https://libraries.minecraft.net/org/test.jar";
        let mirror = "https://bmclapi2.bangbang93.com/maven/org/test.jar";
        let legacy = "https://bmclapi2.bangbang93.com/libraries/org/test.jar";
        assert_eq!(
            bases.download_urls(official, SourcePreference::Mirror, true),
            vec![mirror, legacy, official]
        );
        assert_eq!(
            bases.download_urls(mirror, SourcePreference::Official, false),
            vec![official, mirror, legacy]
        );
        assert_eq!(
            bases.download_urls(official, SourcePreference::Auto, false),
            vec![mirror, legacy, official]
        );
        assert_eq!(
            bases.download_urls(official, SourcePreference::Auto, true),
            vec![official, mirror, legacy]
        );
        let assets = bases.download_urls(
            "https://bmclapi2.bangbang93.com/assets/ab/hash",
            SourcePreference::Official,
            true,
        );
        assert_eq!(
            assets[0],
            "https://resources.download.minecraft.net/ab/hash"
        );
        for url in [
            "https://example.org/file",
            "https://libraries.minecraft.net.evil.test/file",
            "https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader/1.20.1/0.16.14/profile/json",
        ] {
            assert_eq!(
                bases.download_urls(url, SourcePreference::Mirror, false),
                vec![url]
            );
        }
    }

    #[test]
    fn mirrors_known_official_hosts_without_rewriting_third_party_origins() {
        let bases = ApiBases::new(&ApiProvider::BMCLApi);
        for (url, expected) in [
            (
                "https://piston-meta.mojang.com/v1/test.json",
                "https://bmclapi2.bangbang93.com/v1/test.json",
            ),
            (
                "https://piston-data.mojang.com/v1/client.jar",
                "https://bmclapi2.bangbang93.com/v1/client.jar",
            ),
            (
                "https://libraries.minecraft.net/org/test.jar",
                "https://bmclapi2.bangbang93.com/maven/org/test.jar",
            ),
            (
                "https://resources.download.minecraft.net/ab/hash",
                "https://bmclapi2.bangbang93.com/assets/ab/hash",
            ),
            (
                "https://example.org/test.jar?file=1",
                "https://example.org/test.jar?file=1",
            ),
            (
                "https://libraries.minecraft.net.evil.test/a",
                "https://libraries.minecraft.net.evil.test/a",
            ),
        ] {
            assert_eq!(bases.download_url(url), expected);
        }
    }

    #[tokio::test]
    async fn mc_manifest() {
        let mc_api_client = MinecraftApiClient::new(Client::new(), &ApiProvider::Official);
        let manifest: game::McVersionManifest = mc_api_client
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
}
