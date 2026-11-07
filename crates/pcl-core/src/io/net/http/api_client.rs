use crate::io::net::http::api_client::plugins::McPluginReport;
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
    pub fabric_base: &'static str,
    pub legacy_fabric_base: &'static str,
    #[cfg(feature = "community")]
    pub modrinth_base: &'static str,
    #[cfg(feature = "community")]
    pub curseforge_base: &'static str,
}

impl ApiBases {
    pub fn new(provider: &ApiProvider) -> Self {
        match provider {
            ApiProvider::Official => ApiBases {
                meta_base: "https://piston-meta.mojang.com",
                resources_base: "https://resources.download.minecraft.net",
                fabric_base: "https://meta.fabricmc.net/v2/versions/loader",
                legacy_fabric_base: "https://meta.legacyfabric.net/v2/versions/loader",
                #[cfg(feature = "community")]
                modrinth_base: "https://api.modrinth.com/v2",
                #[cfg(feature = "community")]
                curseforge_base: "https://api.curseforge.com/v1",
            },
            ApiProvider::BMCLApi => ApiBases {
                meta_base: "https://bmclapi2.bangbang93.com",
                resources_base: "https://bmclapi2.bangbang93.com/assets",
                fabric_base: "https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader",
                legacy_fabric_base: "https://meta.legacyfabric.net/v2/versions/loader",
                #[cfg(feature = "community")]
                modrinth_base: "https://api.modrinth.com/v2",
                #[cfg(feature = "community")]
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
        if url.starts_with("https://files.minecraftforge.net/maven/net/minecraftforge/forge/index_")
            && url.ends_with(".html")
        {
            return vec![url.into()];
        }
        let forgelike_path = url
            .strip_prefix("https://maven.minecraftforge.net/")
            .or_else(|| url.strip_prefix("https://files.minecraftforge.net/maven/"))
            .or_else(|| url.strip_prefix("https://maven.neoforged.net/releases/"));
        if let Some(path) = forgelike_path {
            let mirror = format!("https://bmclapi2.bangbang93.com/maven/{path}");
            return if matches!(preference, SourcePreference::Official)
                || (matches!(preference, SourcePreference::Auto) && official_fast)
            {
                vec![url.into(), mirror]
            } else {
                vec![mirror, url.into()]
            };
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

impl FabricFamily {
    pub fn installer_url(self, version: &str) -> McApiResult<String> {
        if version.is_empty()
            || version == "."
            || version == ".."
            || !version
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'+'))
        {
            return Err(McApiError::InvalidMetadata(
                "Invalid installer version".into(),
            ));
        }
        let (host, group) = match self {
            Self::Fabric => ("maven.fabricmc.net", "fabricmc"),
            Self::Legacyfabric => ("maven.legacyfabric.net", "legacyfabric"),
        };
        Ok(format!(
            "https://{host}/net/{group}/fabric-installer/{version}/fabric-installer-{version}.jar"
        ))
    }
}

fn fabric_metadata_sources(
    base: &'static str,
    preference: crate::io::download::resources::SourcePreference,
) -> Vec<&'static str> {
    let official = "https://meta.fabricmc.net/v2/versions/loader";
    let mirror = "https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader";
    if base != official && base != mirror {
        return vec![base];
    }
    if matches!(
        preference,
        crate::io::download::resources::SourcePreference::Official
    ) {
        vec![official, mirror]
    } else {
        vec![mirror, official]
    }
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
}

pub(crate) fn neoforge_game_version(version: &str) -> Option<String> {
    if version.starts_with("1.20.1-") {
        return Some("1.20.1".into());
    }
    let core = version.split(['-', '+']).next()?;
    let parts: Vec<_> = core.split('.').collect();
    if parts.len() < 3 || parts.len() > 4 {
        return None;
    }
    let major: u32 = parts[0].parse().ok()?;
    if major == 0 {
        parts.last()?.parse::<u32>().ok()?;
        return Some(parts[1].to_owned());
    }
    let minor: u32 = parts[1].parse().ok()?;
    let patch: u32 = parts[2].parse().ok()?;
    if let Some(revision) = parts.get(3) {
        revision.parse::<u32>().ok()?;
    }
    let mut game = if major >= 24 {
        format!(
            "{major}.{minor}{}",
            if patch > 0 {
                format!(".{patch}")
            } else {
                String::new()
            }
        )
    } else {
        format!(
            "1.{major}{}",
            if minor > 0 {
                format!(".{minor}")
            } else {
                String::new()
            }
        )
    };
    if let Some((_, suffix)) = version.split_once('+') {
        game.push('-');
        game.push_str(suffix);
    }
    Some(game)
}

fn parse_neoforge_versions(
    value: serde_json::Value,
    game: &str,
) -> McApiResult<Vec<McPluginReport>> {
    let versions = value
        .get("versions")
        .or_else(|| value.get("files"))
        .and_then(|v| v.as_array())
        .ok_or_else(|| McApiError::InvalidMetadata("Invalid NeoForge versions response".into()))?;
    let mut result = Vec::new();
    for item in versions {
        if item
            .get("type")
            .and_then(|v| v.as_str())
            .is_some_and(|kind| kind != "DIRECTORY")
        {
            continue;
        }
        let Some(raw) = item
            .as_str()
            .or_else(|| item.get("version").and_then(|v| v.as_str()))
            .or_else(|| item.get("name").and_then(|v| v.as_str()))
        else {
            continue;
        };
        if !raw
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'+' | b'_'))
            || raw == "1.20.1-47.1.82"
            || neoforge_game_version(raw).as_deref() != Some(game)
        {
            continue;
        }
        let version = raw.strip_prefix("1.20.1-").unwrap_or(raw);
        result.push(McPluginReport {
            version: version.to_owned(),
            stable: Some(!raw.contains("beta") && !raw.contains("alpha")),
            release_time: None,
        });
    }
    result.sort_by(|a, b| {
        let numbers = |version: &str| {
            version
                .split(['-', '+'])
                .next()
                .unwrap_or_default()
                .split('.')
                .map(|part| part.parse::<u32>().unwrap_or(0))
                .collect::<Vec<_>>()
        };
        numbers(&b.version)
            .cmp(&numbers(&a.version))
            .then_with(|| b.stable.cmp(&a.stable))
            .then_with(|| b.version.cmp(&a.version))
    });
    Ok(result)
}

fn parse_fabric_installers(
    value: serde_json::Value,
    family: FabricFamily,
) -> McApiResult<Vec<McPluginReport>> {
    let entries = value
        .as_array()
        .ok_or_else(|| McApiError::InvalidMetadata("Invalid installer list".into()))?;
    let mut installers = Vec::new();
    for entry in entries {
        let version = entry
            .get("version")
            .and_then(|v| v.as_str())
            .ok_or_else(|| McApiError::InvalidMetadata("Missing installer version".into()))?;
        let url = family.installer_url(version)?;
        if entry.get("url").and_then(|v| v.as_str()) != Some(url.as_str()) {
            return Err(McApiError::InvalidMetadata("Invalid installer URL".into()));
        }
        installers.push(McPluginReport {
            version: version.into(),
            stable: entry.get("stable").and_then(|v| v.as_bool()),
            release_time: None,
        });
    }
    Ok(installers)
}

pub struct MinecraftApiClient {
    transport: crate::io::download::Downloader,
    api_bases: RwLock<ApiBases>,
    cache: DashMap<String, (Instant, serde_json::Value)>,
    ttl: Duration,
}

impl MinecraftApiClient {
    pub async fn get_fabric_installers(
        &self,
        family: FabricFamily,
    ) -> McApiResult<Vec<McPluginReport>> {
        let bases = self.api_bases.read().await;
        let base = match family {
            FabricFamily::Fabric => bases.fabric_base,
            FabricFamily::Legacyfabric => bases.legacy_fabric_base,
        };
        drop(bases);
        let mut last_error = None;
        for source in
            fabric_metadata_sources(base, self.transport.resources().settings().version_source)
        {
            let url = format!(
                "{}/installer",
                source.strip_suffix("/loader").unwrap_or(source)
            );
            match self
                .get::<serde_json::Value>(&url, true)
                .await
                .and_then(|value| parse_fabric_installers(value, family))
            {
                Ok(installers) => return Ok(installers),
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.unwrap_or_else(|| {
            McApiError::InvalidMetadata("No installer metadata source available".into())
        }))
    }
    /// Create a new MinecraftApiClient
    pub fn new(transport: crate::io::download::Downloader, api_provider: &ApiProvider) -> Self {
        Self {
            transport,
            api_bases: RwLock::new(ApiBases::new(api_provider)),
            cache: DashMap::new(),
            ttl: Duration::from_secs(60 * 5),
        }
    }

    pub fn replace_clients(&self, client: Client, http1: Client) {
        self.transport.replace_clients(client, http1);
    }

    pub fn http_client(&self) -> Client {
        self.transport.http_client()
    }

    pub fn resources(&self) -> &std::sync::Arc<crate::io::download::resources::DownloadResources> {
        self.transport.resources()
    }

    pub async fn downloader(&self) -> crate::io::download::Downloader {
        self.transport.for_bases(self.api_bases_async().await)
    }

    #[cfg(test)]
    pub(crate) fn with_test_bases(bases: ApiBases) -> Self {
        let mut client = Self::new(
            crate::io::download::Downloader::default(),
            &ApiProvider::Official,
        );
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
        use crate::io::download::resources::SourcePreference;
        use futures_util::StreamExt;
        use std::sync::atomic::Ordering;
        let settings = self.resources().settings();
        let manifest = url.ends_with(game::VERSION_MANIFEST_ENDPOINT);
        let preference = if manifest {
            settings.version_source
        } else {
            settings.file_source
        };
        let urls = self.api_bases.read().await.download_urls(
            url,
            preference,
            manifest || self.resources().official_fast.load(Ordering::Relaxed),
        );
        let mut last_error = None;
        for actual_url in urls {
            let official_manifest =
                manifest && actual_url.starts_with("https://piston-meta.mojang.com/");
            let result = async {
                let _lease = self.resources().acquire().await;
                let started = Instant::now();
                let timeout = Duration::from_secs(
                    if official_manifest && matches!(preference, SourcePreference::Auto) {
                        5
                    } else {
                        15
                    },
                );
                let response = self
                    .transport
                    .send_request(settings.http_mode, false, &actual_url, None, Some(timeout))
                    .await?
                    .error_for_status()?;
                let mut stream = response.bytes_stream();
                let mut bytes = Vec::new();
                while let Some(chunk) = stream.next().await {
                    let chunk = chunk?;
                    self.resources().record_bytes(chunk.len());
                    self.resources().throttle(chunk.len()).await;
                    bytes.extend_from_slice(&chunk);
                }
                let value = validate(&bytes)?;
                if official_manifest {
                    self.resources().official_fast.store(
                        started.elapsed() < Duration::from_secs(4),
                        Ordering::Relaxed,
                    );
                }
                Ok(value)
            }
            .await;
            if official_manifest && result.is_err() {
                self.resources()
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
        if allow_from_cache && let Some(data) = self.cache_hit(url) {
            return Ok(data);
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
        Ok(self
            .forge_versions(version_id, None)
            .await?
            .into_iter()
            .map(|entry| McPluginReport {
                version: entry.version,
                stable: None,
                release_time: entry.release_time,
            })
            .collect())
    }

    pub async fn get_neoforge_versions(
        &self,
        version_id: &str,
    ) -> McApiResult<Vec<McPluginReport>> {
        crate::io::files::validate_file_name(version_id)
            .map_err(|error| McApiError::InvalidMetadata(error.to_string()))?;
        let package = if version_id == "1.20.1" {
            "forge"
        } else {
            "neoforge"
        };
        let official = format!(
            "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/{package}"
        );
        let mirror = format!(
            "https://bmclapi2.bangbang93.com/neoforge/meta/api/maven/details/releases/net/neoforged/{package}"
        );
        let urls = match self.resources().settings().version_source {
            crate::io::download::resources::SourcePreference::Official => [official, mirror],
            _ => [mirror, official],
        };
        let mut last_error = None;
        for url in urls {
            match self
                .get::<serde_json::Value>(&url, true)
                .await
                .and_then(|v| parse_neoforge_versions(v, version_id))
            {
                Ok(versions) => return Ok(versions),
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.expect("NeoForge has two metadata sources"))
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
        let mut last_error = None;
        let mut fabric_versions = None;
        for source in
            fabric_metadata_sources(base, self.transport.resources().settings().version_source)
        {
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
        use crate::io::download::resources::SourcePreference;
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
            match self.resources().settings().file_source {
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

    #[cfg(feature = "community")]
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
    #[cfg(feature = "community")]
    pub async fn fetch_with_curseforge<T: DeserializeOwned + Serialize>(
        &self,
        endpoint: &str,
        api_key: &str,
    ) -> McApiResult<T> {
        if api_key.trim().is_empty() {
            return Err(McApiError::MissingApiKey("CurseForge"));
        }
        let curseforge_base = self.api_bases.read().await.curseforge_base;
        let url = format!("{curseforge_base}/{endpoint}");
        use sha2::Digest;
        let cache_key = format!(
            "curseforge:{:x}:{url}",
            sha2::Sha256::digest(api_key.as_bytes())
        );
        let now = Instant::now();
        if let Some(data) = self.cache_hit(&cache_key) {
            return Ok(data);
        }
        let client = self.http_client();
        let data: T = client
            .get(&url)
            .header("x-api-key", api_key)
            .timeout(Duration::from_secs(15))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        self.cache_store(&cache_key, now, &data);
        Ok(data)
    }

    pub async fn get_bytes(&self, url: &str) -> McApiResult<Vec<u8>> {
        self.fetch_checked(url, |bytes| Ok(bytes.to_vec())).await
    }
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn independent_clients_isolate_transport_credentials_and_quotas() {
        use crate::io::download::resources::DownloadOptions;
        use axum::{Json, Router, http::HeaderMap, routing::get};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base: &'static str =
            Box::leak(format!("http://{}", listener.local_addr().unwrap()).into_boxed_str());
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new().route(
                    "/echo",
                    get(|headers: HeaderMap| async move {
                        Json(serde_json::json!({
                            "agent": headers.get("user-agent").unwrap().to_str().unwrap(),
                            "key": headers.get("x-api-key").map(|v| v.to_str().unwrap()),
                        }))
                    }),
                ),
            )
            .await
            .unwrap();
        });
        let clients = |agent: &str| {
            (
                Client::builder()
                    .no_proxy()
                    .user_agent(agent)
                    .build()
                    .unwrap(),
                Client::builder()
                    .no_proxy()
                    .user_agent(agent)
                    .http1_only()
                    .build()
                    .unwrap(),
            )
        };
        let make = |agent: &str| {
            let mut bases = ApiBases::new(&ApiProvider::Official);
            bases.meta_base = base;
            #[cfg(feature = "community")]
            {
                bases.modrinth_base = base;
                bases.curseforge_base = base;
            }
            let api = MinecraftApiClient::with_test_bases(bases);
            let (client, http1) = clients(agent);
            api.replace_clients(client, http1);
            api
        };
        let first = make("FirstLauncher");
        let second = make("SecondLauncher");
        let download = first.downloader().await;
        first
            .resources()
            .configure(DownloadOptions {
                max_connections: 1,
                ..Default::default()
            })
            .unwrap();
        let lease = download.resources().acquire().await;
        assert!(
            tokio::time::timeout(Duration::from_millis(10), first.resources().acquire())
                .await
                .is_err()
        );
        let other_lease = second.resources().acquire().await;
        assert_eq!(second.resources().settings().max_connections, 64);
        drop((lease, other_lease));
        let echo = format!("{base}/echo");
        let metadata: serde_json::Value =
            serde_json::from_slice(&first.get_bytes(&echo).await.unwrap()).unwrap();
        assert_eq!(
            metadata,
            serde_json::json!({"agent": "FirstLauncher", "key": null})
        );
        #[cfg(feature = "community")]
        {
            assert!(matches!(
                first
                    .fetch_with_curseforge::<serde_json::Value>("echo", "")
                    .await,
                Err(McApiError::MissingApiKey("CurseForge"))
            ));
            let reply: serde_json::Value = first.fetch_with_modrinth("echo").await.unwrap();
            assert_eq!(reply["key"], serde_json::Value::Null);
            let reply: serde_json::Value = first
                .fetch_with_curseforge("echo", "first-key")
                .await
                .unwrap();
            assert_eq!(
                reply,
                serde_json::json!({"agent": "FirstLauncher", "key": "first-key"})
            );
            let reply: serde_json::Value = first
                .fetch_with_curseforge("echo", "another-key")
                .await
                .unwrap();
            assert_eq!(reply["key"], "another-key");
            let reply: serde_json::Value = first.get(&echo, true).await.unwrap();
            assert_eq!(reply["key"], serde_json::Value::Null);
            let reply: serde_json::Value = second
                .fetch_with_curseforge("echo", "second-key")
                .await
                .unwrap();
            assert_eq!(
                reply,
                serde_json::json!({"agent": "SecondLauncher", "key": "second-key"})
            );
        }
        let (client, http1) = clients("UpdatedLauncher");
        first.replace_clients(client, http1);
        let reply: serde_json::Value = download
            .http_client()
            .get(&echo)
            .send()
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
        assert_eq!(
            reply,
            serde_json::json!({"agent": "UpdatedLauncher", "key": null})
        );
        let reply: serde_json::Value =
            serde_json::from_slice(&second.get_bytes(&echo).await.unwrap()).unwrap();
        assert_eq!(
            reply,
            serde_json::json!({"agent": "SecondLauncher", "key": null})
        );
        server.abort();
    }

    #[test]
    fn neoforge_versions_match_ce_game_mapping() {
        use super::*;
        let versions = serde_json::json!({"versions": [
            "1.20.1-47.1.82", "1.20.1-47.1.99", "20.4.30-beta",
            "0.25w14craftmine.3-beta", "26.1.0.0-alpha.1+snapshot-1"
        ]});
        for (game, expected) in [
            ("1.20.1", "47.1.99"),
            ("1.20.4", "20.4.30-beta"),
            ("25w14craftmine", "0.25w14craftmine.3-beta"),
            ("26.1-snapshot-1", "26.1.0.0-alpha.1+snapshot-1"),
        ] {
            let entries = parse_neoforge_versions(versions.clone(), game).unwrap();
            assert_eq!(entries.len(), 1, "{game}");
            assert_eq!(entries[0].version, expected);
        }
        let sorted = parse_neoforge_versions(
            serde_json::json!({"versions": ["20.4.9", "20.4.11-beta", "20.4.10"]}),
            "1.20.4",
        )
        .unwrap();
        assert_eq!(
            sorted
                .iter()
                .map(|v| v.version.as_str())
                .collect::<Vec<_>>(),
            ["20.4.11-beta", "20.4.10", "20.4.9"]
        );
        assert!(parse_neoforge_versions(serde_json::json!({}), "1.20.1").is_err());
        let mirror = serde_json::json!({"files": [
            {"name":"20.4.30-beta", "type":"DIRECTORY"},
            {"name":"20.4.31", "type":"FILE"}
        ]});
        let mirror = parse_neoforge_versions(mirror, "1.20.4").unwrap();
        assert_eq!(mirror.len(), 1);
        assert_eq!(mirror[0].version, "20.4.30-beta");
    }

    #[test]
    fn fabric_installer_metadata_requires_matching_official_url() {
        use super::*;
        let url = FabricFamily::Legacyfabric.installer_url("1.1.0+1").unwrap();
        let good = serde_json::json!([{ "version": "1.1.0+1", "url": url, "stable": false }]);
        let entries = parse_fabric_installers(good, FabricFamily::Legacyfabric).unwrap();
        assert_eq!(entries[0].version, "1.1.0+1");
        assert!(
            parse_fabric_installers(
                serde_json::json!([{ "version": "1.0", "url": "https://evil.test/a.jar" }]),
                FabricFamily::Fabric
            )
            .is_err()
        );
        assert!(
            parse_fabric_installers(serde_json::json!([{ "url": "x" }]), FabricFamily::Fabric)
                .is_err()
        );
        assert!(FabricFamily::Fabric.installer_url("../1.0").is_err());
    }
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
        assert_eq!(
            bases.download_urls(
                "https://maven.neoforged.net/releases/net/example/lib.jar",
                SourcePreference::Mirror,
                false
            ),
            [
                "https://bmclapi2.bangbang93.com/maven/net/example/lib.jar",
                "https://maven.neoforged.net/releases/net/example/lib.jar"
            ]
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
}
