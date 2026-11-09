use super::HttpTransport;
use super::resources::SourceKind;
use crate::io::net::http::api_client::plugins::McPluginReport;
#[cfg(test)]
use crate::minecraft::sources::ApiEndpoints;
use crate::minecraft::{mod_loader::FabricFamily, sources::Sources};
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
    transport: HttpTransport,
    pub(super) sources: Sources,
    cache: DashMap<String, (Instant, serde_json::Value)>,
    ttl: Duration,
}

impl MinecraftApiClient {
    pub async fn get_fabric_installers(
        &self,
        family: FabricFamily,
    ) -> McApiResult<Vec<McPluginReport>> {
        let mut last_error = None;
        for source in self.sources.fabric(family, SourceKind::VersionMetadata) {
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
    pub fn new(transport: HttpTransport) -> Self {
        Self {
            sources: Sources::new(transport.resources().clone()),
            transport,
            cache: DashMap::new(),
            ttl: Duration::from_secs(60 * 5),
        }
    }

    /// Configure CurseForge credentials for this client's resolved official sources.
    #[cfg(feature = "community")]
    pub fn with_curseforge_api_key(mut self, key: Option<&str>) -> Self {
        self.sources.set_curseforge_api_key(key);
        self.cache.clear();
        self
    }

    pub fn transport(&self) -> &HttpTransport {
        &self.transport
    }

    pub fn replace_clients(&self, client: Client, http1: Client) {
        self.transport.replace_clients(client, http1);
    }

    pub fn http_client(&self) -> Client {
        self.transport.http_client()
    }

    pub fn resources(&self) -> &std::sync::Arc<crate::io::net::http::resources::DownloadResources> {
        self.transport.resources()
    }

    pub fn downloader(&self) -> crate::io::download::Downloader {
        let sources = self.sources.clone();
        crate::io::download::Downloader::new(self.transport.clone())
            .with_sources(move |url, kind| sources.resolve(url, kind))
    }

    #[cfg(test)]
    pub(crate) fn with_test_endpoints(endpoints: ApiEndpoints) -> Self {
        Self::with_test_transport(endpoints, HttpTransport::default())
    }

    #[cfg(test)]
    pub(crate) fn with_test_transport(endpoints: ApiEndpoints, transport: HttpTransport) -> Self {
        let mut client = Self::new(transport);
        client.sources.endpoints = endpoints;
        client
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
        use futures_util::StreamExt;
        let settings = self.resources().settings();
        let targets = self.sources.resolve(url, SourceKind::File);
        let mut last_error = None;
        for target in targets {
            let actual_url = &target.url;
            let _lease = self.resources().acquire().await;
            let started = Instant::now();
            let result = async {
                let response = self
                    .transport
                    .send_request(
                        settings.http_mode,
                        false,
                        &target,
                        None,
                        Some(self.sources.timeout(actual_url)),
                    )
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
                validate(&bytes)
            }
            .await;
            self.sources
                .record_response(actual_url, started.elapsed(), result.is_ok());
            match result {
                Ok(value) => return Ok(value),
                Err(error) => {
                    log::warn!("Metadata source failed for {actual_url}: {error:?}");
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
        let mut last_error = None;
        for url in self.sources.neoforge(package) {
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
        let mut last_error = None;
        let mut fabric_versions = None;
        for source in self.sources.fabric(family, SourceKind::VersionMetadata) {
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
        for value in [version_id, loader] {
            crate::io::files::validate_file_name(value)
                .map_err(|error| McApiError::InvalidMetadata(error.to_string()))?;
        }
        let mut last_error = None;
        for base in self.sources.fabric(family, SourceKind::File) {
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
        let modrinth_base = self.sources.endpoints.modrinth_base;
        let url = format!("{modrinth_base}/{endpoint}");
        self.get(&url, true).await
    }

    /// 源解析按实际目标附带 CurseForge 凭据；镜像与回退由下载源设置决定。
    #[cfg(feature = "community")]
    pub async fn fetch_with_curseforge<T: DeserializeOwned + Serialize>(
        &self,
        endpoint: &str,
    ) -> McApiResult<T> {
        let curseforge_base = self.sources.endpoints.curseforge_base;
        self.get(&format!("{curseforge_base}/{endpoint}"), true)
            .await
    }

    pub async fn get_bytes(&self, url: &str) -> McApiResult<Vec<u8>> {
        self.fetch_checked(url, |bytes| Ok(bytes.to_vec())).await
    }
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "community")]
    #[test]
    fn curseforge_credentials_follow_resolved_destinations() {
        use crate::io::net::http::resources::{DownloadOptions, SourcePreference};
        let transport = HttpTransport::default();
        let first = MinecraftApiClient::new(transport.clone())
            .with_curseforge_api_key(Some("  first-key  "));
        let second =
            MinecraftApiClient::new(transport.clone()).with_curseforge_api_key(Some("second-key"));
        let plain = MinecraftApiClient::new(transport);
        first
            .resources()
            .configure(DownloadOptions {
                community_source: SourcePreference::Mirror,
                ..Default::default()
            })
            .unwrap();
        for official in [
            "https://api.curseforge.com/v1/mods/1",
            "https://edge.forgecdn.net/files/a.jar",
            "https://mediafilez.forgecdn.net/files/a.jar",
            "https://cursecdn.com/files/a.jar",
            "https://media.cursecdn.com/files/a.jar",
        ] {
            let targets = first.sources.resolve(official, SourceKind::File);
            let downloaded = first
                .downloader()
                .resolve_sources(official, SourceKind::File);
            for (target, download) in targets.iter().zip(&downloaded) {
                assert_eq!(target.url, download.url);
                assert_eq!(target.headers, download.headers);
                if target.url == official {
                    assert_eq!(target.headers["x-api-key"], "first-key");
                    assert!(target.headers["x-api-key"].is_sensitive());
                } else {
                    assert!(target.headers.is_empty());
                }
            }
            let other = second.sources.resolve(official, SourceKind::File);
            assert_eq!(other.last().unwrap().headers["x-api-key"], "second-key");
            assert!(
                plain
                    .sources
                    .resolve(official, SourceKind::File)
                    .iter()
                    .all(|t| t.headers.is_empty())
            );
        }
        for url in [
            "https://mod.mcimirror.top/curseforge/v1/mods/1",
            "https://api.modrinth.com/v2/project/test",
            "https://cdn.modrinth.com/data/a.jar",
            "https://www.curseforge.com/minecraft/mc-mods/jei",
            "https://api.curseforge.com.evil.test/v1/mods/1",
            "https://evilcursecdn.com/files/a.jar",
            "http://edge.forgecdn.net/files/a.jar",
        ] {
            assert!(
                first
                    .sources
                    .resolve(url, SourceKind::File)
                    .iter()
                    .all(|t| t.headers.is_empty()),
                "{url}"
            );
        }
        for key in [None, Some("   "), Some("bad\nkey")] {
            let api =
                MinecraftApiClient::new(HttpTransport::default()).with_curseforge_api_key(key);
            assert!(
                api.sources
                    .resolve("https://api.curseforge.com/v1/mods/1", SourceKind::File)
                    .iter()
                    .all(|t| t.headers.is_empty())
            );
        }
    }

    #[test]
    fn source_updates_reach_existing_downloaders_without_crossing_services() {
        use crate::io::net::http::resources::{DownloadOptions, SourceKind, SourcePreference};
        let api = MinecraftApiClient::new(HttpTransport::default());
        let downloader = api.downloader();
        let library = "https://libraries.minecraft.net/org/test.jar";
        let manifest = format!(
            "https://piston-meta.mojang.com/{}",
            game::VERSION_MANIFEST_ENDPOINT
        );
        assert_eq!(
            downloader.resolve_sources(library, SourceKind::File)[0].url,
            library
        );
        api.sources
            .record_response(&manifest, Duration::from_secs(5), false);
        assert!(
            downloader.resolve_sources(library, SourceKind::File)[0]
                .url
                .starts_with("https://bmclapi2.bangbang93.com/")
        );
        api.sources
            .record_response("https://example.org/metadata", Duration::ZERO, true);
        assert!(
            downloader.resolve_sources(library, SourceKind::File)[0]
                .url
                .starts_with("https://bmclapi2.bangbang93.com/")
        );
        api.sources.record_response(&manifest, Duration::ZERO, true);
        assert_eq!(
            downloader.resolve_sources(library, SourceKind::File)[0].url,
            library
        );
        api.resources()
            .configure(DownloadOptions {
                file_source: SourcePreference::Mirror,
                community_source: SourcePreference::Official,
                ..Default::default()
            })
            .unwrap();
        assert!(
            downloader.resolve_sources(library, SourceKind::File)[0]
                .url
                .starts_with("https://bmclapi2.bangbang93.com/")
        );
        #[cfg(feature = "community")]
        for official in [
            "https://api.curseforge.com/v1/mods/1",
            "https://api.modrinth.com/v2/project/test",
        ] {
            assert_eq!(
                downloader.resolve_sources(official, SourceKind::File)[0].url,
                official
            );
            assert_eq!(
                api.sources.resolve(official, SourceKind::File)[0].url,
                official
            );
        }
    }

    #[tokio::test]
    async fn clients_share_transport_resources_and_send_explicit_headers() {
        use crate::io::net::http::resources::DownloadOptions;
        use axum::{Json, Router, http::HeaderMap, routing::get};
        let echo_handler = |headers: HeaderMap| async move {
            Json(serde_json::json!({
                "agent": headers.get("user-agent").unwrap().to_str().unwrap(),
                "key": headers.get("x-api-key").map(|v| v.to_str().unwrap()),
            }))
        };
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base: &'static str =
            Box::leak(format!("http://{}", listener.local_addr().unwrap()).into_boxed_str());
        let server = tokio::spawn(async move {
            axum::serve(
                listener,
                Router::new()
                    .route("/echo", get(echo_handler))
                    .route(
                        "/cf/echo/unavailable",
                        get(|headers: HeaderMap| async move {
                            assert!(!headers.contains_key("x-api-key"));
                            reqwest::StatusCode::FORBIDDEN
                        }),
                    )
                    .route("/cf/echo", get(echo_handler)),
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
            let mut bases = ApiEndpoints::default();
            bases.meta_base = base;
            #[cfg(feature = "community")]
            {
                bases.modrinth_base = base;
                bases.curseforge_base = Box::leak(format!("{base}/cf").into_boxed_str());
            }
            let api = MinecraftApiClient::with_test_transport(bases, HttpTransport::default());
            let (client, http1) = clients(agent);
            api.replace_clients(client, http1);
            api
        };
        let first = make("FirstLauncher");
        #[cfg(feature = "community")]
        let first = first.with_curseforge_api_key(Some("first-key"));
        let second = make("SecondLauncher");
        let download = crate::io::download::Downloader::new(first.transport().clone());
        let provider_download = first.downloader();
        first
            .resources()
            .configure(DownloadOptions {
                max_connections: 1,
                ..Default::default()
            })
            .unwrap();
        let lease = provider_download.resources().acquire().await;
        assert!(
            tokio::time::timeout(Duration::from_millis(10), download.resources().acquire())
                .await
                .is_err()
        );
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
            // 自定义 HTTP 地址即使作为 CurseForge endpoint，也不能收到官方密钥。
            let reply: serde_json::Value = first.fetch_with_curseforge("echo").await.unwrap();
            assert_eq!(reply["key"], serde_json::Value::Null);
            let reply: serde_json::Value = first.fetch_with_modrinth("echo").await.unwrap();
            assert_eq!(reply["key"], serde_json::Value::Null);
        }
        let (client, http1) = clients("UpdatedLauncher");
        first.replace_clients(client, http1);
        let output = tempfile::tempdir().unwrap();
        let path = output.path().join("echo.json");
        download
            .with_sources(|url, _| {
                let mut target = super::super::RequestTarget::new(url);
                target
                    .headers
                    .insert("x-api-key", "explicit-key".parse().unwrap());
                vec![
                    super::super::RequestTarget::new(format!("{url}/unavailable")),
                    target,
                ]
            })
            .download(
                &game::DownloadFile {
                    url: format!("{base}/cf/echo"),
                    path: None,
                    size: 0,
                    sha1: String::new(),
                },
                &path,
            )
            .await
            .unwrap();
        let reply: serde_json::Value =
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap();
        assert_eq!(
            reply,
            serde_json::json!({"agent": "UpdatedLauncher", "key": "explicit-key"})
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
}
