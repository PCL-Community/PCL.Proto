//! Service-specific official endpoints, mirror mappings and source ordering.
use super::version::VERSION_MANIFEST_ENDPOINT;
use crate::io::net::http::RequestTarget;
use crate::io::net::http::resources::{
    DownloadOptions, DownloadResources, SourceKind, SourcePreference,
};
use crate::minecraft::mod_loader::FabricFamily;
use std::{
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

const MOJANG_META: &str = "https://piston-meta.mojang.com";
const BMCL: &str = "https://bmclapi2.bangbang93.com";

pub(crate) const AUTHLIB_METADATA: &str = "https://authlib-injector.yushi.moe/artifact/latest.json";

/// Authlib keeps official-first fallback independently of Minecraft's source preference.
pub(crate) fn authlib_sources(url: &str) -> Vec<String> {
    const OFFICIAL: &str = "https://authlib-injector.yushi.moe/";
    const MIRROR: &str = "https://bmclapi2.bangbang93.com/mirrors/authlib-injector/";
    match url
        .strip_prefix(OFFICIAL)
        .or_else(|| url.strip_prefix(MIRROR))
    {
        Some(path) => vec![format!("{OFFICIAL}{path}"), format!("{MIRROR}{path}")],
        None => vec![url.to_owned()],
    }
}

#[derive(Clone)]
pub(crate) struct ApiEndpoints {
    pub meta_base: &'static str,
    pub resources_base: &'static str,
    pub fabric_base: &'static str,
    pub legacy_fabric_base: &'static str,
    #[cfg(feature = "community")]
    pub modrinth_base: &'static str,
    #[cfg(feature = "community")]
    pub curseforge_base: &'static str,
}

impl Default for ApiEndpoints {
    fn default() -> Self {
        Self {
            meta_base: MOJANG_META,
            resources_base: "https://resources.download.minecraft.net",
            fabric_base: "https://meta.fabricmc.net/v2/versions/loader",
            legacy_fabric_base: "https://meta.legacyfabric.net/v2/versions/loader",
            #[cfg(feature = "community")]
            modrinth_base: "https://api.modrinth.com/v2",
            #[cfg(feature = "community")]
            curseforge_base: "https://api.curseforge.com/v1",
        }
    }
}

/// Mirror only Mojang-owned download hosts; third-party URLs retain their origin.
fn mojang_url(url: &str, meta_base: &str, resources_base: &str) -> String {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return url.into();
    };
    let prefix = match parsed.host_str() {
        Some(
            "piston-meta.mojang.com"
            | "launchermeta.mojang.com"
            | "piston-data.mojang.com"
            | "launcher.mojang.com",
        ) => meta_base.to_owned(),
        Some("libraries.minecraft.net") => format!("{}/maven", meta_base),
        Some("resources.download.minecraft.net") => resources_base.to_owned(),
        _ => return url.into(),
    };
    let mut rewritten = format!("{prefix}{}", parsed.path());
    if let Some(query) = parsed.query() {
        rewritten.push('?');
        rewritten.push_str(query);
    }
    rewritten
}

#[derive(Clone)]
pub(crate) struct Sources {
    pub endpoints: ApiEndpoints,
    resources: Arc<DownloadResources>,
    official_fast: Arc<AtomicBool>,
    #[cfg(feature = "community")]
    curseforge_api_key: Option<reqwest::header::HeaderValue>,
}

impl Sources {
    pub fn new(resources: Arc<DownloadResources>) -> Self {
        Self {
            endpoints: ApiEndpoints::default(),
            resources,
            official_fast: Arc::new(AtomicBool::new(true)),
            #[cfg(feature = "community")]
            curseforge_api_key: None,
        }
    }

    #[cfg(feature = "community")]
    pub fn set_curseforge_api_key(&mut self, key: Option<&str>) {
        self.curseforge_api_key = key.and_then(|key| {
            let key = key.trim();
            if key.is_empty() {
                return None;
            }
            let mut value = reqwest::header::HeaderValue::from_bytes(key.as_bytes()).ok()?;
            value.set_sensitive(true);
            Some(value)
        });
    }

    pub fn resolve(&self, url: &str, kind: SourceKind) -> Vec<RequestTarget> {
        let manifest = url.ends_with(VERSION_MANIFEST_ENDPOINT);
        let kind = if manifest {
            SourceKind::VersionMetadata
        } else {
            kind
        };
        let fast =
            manifest || (kind == SourceKind::File && self.official_fast.load(Ordering::Relaxed));
        source_urls(&self.endpoints, url, kind, &self.resources.settings(), fast)
            .into_iter()
            .map(|url| {
                let target = RequestTarget::new(url);
                #[cfg(feature = "community")]
                let mut target = target;
                #[cfg(feature = "community")]
                if let Some(key) = &self.curseforge_api_key
                    && let Ok(url) = reqwest::Url::parse(&target.url)
                    && url.scheme() == "https"
                    && url.host_str().is_some_and(|host| {
                        matches!(
                            host,
                            "api.curseforge.com" | "edge.forgecdn.net" | "mediafilez.forgecdn.net"
                        ) || host == "cursecdn.com"
                            || host.ends_with(".cursecdn.com")
                    })
                {
                    target.headers.insert("x-api-key", key.clone());
                }
                target
            })
            .collect()
    }

    pub fn timeout(&self, url: &str) -> Duration {
        Duration::from_secs(
            if official_manifest(url)
                && matches!(
                    self.resources.settings().version_source,
                    SourcePreference::Auto
                )
            {
                5
            } else {
                15
            },
        )
    }

    pub fn record_response(&self, url: &str, elapsed: Duration, success: bool) {
        if official_manifest(url) {
            self.official_fast.store(
                success && elapsed < Duration::from_secs(4),
                Ordering::Relaxed,
            );
        }
    }

    pub fn fabric(&self, family: FabricFamily, kind: SourceKind) -> Vec<&'static str> {
        let configured = match family {
            FabricFamily::Fabric => self.endpoints.fabric_base,
            FabricFamily::Legacyfabric => self.endpoints.legacy_fabric_base,
        };
        let official = "https://meta.fabricmc.net/v2/versions/loader";
        let mirror = "https://bmclapi2.bangbang93.com/fabric-meta/v2/versions/loader";
        if configured != official && configured != mirror {
            return vec![configured];
        }
        Vec::from(metadata_sources(
            [official, mirror],
            self.resources.settings().preference(kind),
        ))
    }

    pub fn forge(&self, game: &str) -> [(bool, String); 2] {
        let game = game.replace('-', "_");
        let official = format!(
            "https://files.minecraftforge.net/maven/net/minecraftforge/forge/index_{game}.html"
        );
        let mirror = format!("{BMCL}/forge/minecraft/{game}");
        metadata_sources(
            [(false, official), (true, mirror)],
            self.resources.settings().version_source,
        )
    }

    pub fn neoforge(&self, package: &str) -> [String; 2] {
        let official = format!(
            "https://maven.neoforged.net/api/maven/versions/releases/net/neoforged/{package}"
        );
        let mirror =
            format!("{BMCL}/neoforge/meta/api/maven/details/releases/net/neoforged/{package}");
        metadata_sources([official, mirror], self.resources.settings().version_source)
    }
}

fn metadata_sources<T>(mut sources: [T; 2], preference: SourcePreference) -> [T; 2] {
    if !matches!(preference, SourcePreference::Official) {
        sources.reverse();
    }
    sources
}

fn official_manifest(url: &str) -> bool {
    url.starts_with("https://piston-meta.mojang.com/") && url.ends_with(VERSION_MANIFEST_ENDPOINT)
}

fn source_urls(
    endpoints: &ApiEndpoints,
    url: &str,
    kind: SourceKind,
    options: &DownloadOptions,
    official_fast: bool,
) -> Vec<String> {
    let preference = options.preference(kind);
    if endpoints.meta_base != MOJANG_META {
        let overridden = mojang_url(url, endpoints.meta_base, endpoints.resources_base);
        if overridden != url {
            return vec![overridden];
        }
    }
    // Each community service retains its own official origin.
    #[cfg(feature = "community")]
    if let Some(mirror) = mod_mirror_url(url) {
        return community_sources(options.community_source, url, mirror);
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
    let mirror = mojang_url(&canonical, BMCL, "https://bmclapi2.bangbang93.com/assets");
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

/// CurseForge / Modrinth 的 mcim 镜像地址；未覆盖的主机返回 `None`。
///
/// API 主机附加服务路径前缀，文件主机直接换域名；失败时回退官方源。
/// `mediafilez.forgecdn.net` 按 mcim 的说明不做映射。
#[cfg(feature = "community")]
fn mod_mirror_url(url: &str) -> Option<String> {
    const MIRROR: &str = "https://mod.mcimirror.top";
    let parsed = reqwest::Url::parse(url).ok()?;
    if parsed.scheme() != "https" {
        return None;
    }
    let prefix = match parsed.host_str()? {
        "api.curseforge.com" => format!("{MIRROR}/curseforge"),
        "api.modrinth.com" | "staging-api.modrinth.com" => format!("{MIRROR}/modrinth"),
        "edge.forgecdn.net" | "cdn.modrinth.com" => MIRROR.to_owned(),
        _ => return None,
    };
    let mut mirrored = format!("{prefix}{}", parsed.path());
    if let Some(query) = parsed.query() {
        mirrored.push('?');
        mirrored.push_str(query);
    }
    Some(mirrored)
}

/// 按偏好排列官方源与镜像源：镜像源失败时下载器会回退到官方源。
#[cfg(feature = "community")]
fn community_sources(preference: SourcePreference, official: &str, mirror: String) -> Vec<String> {
    match preference {
        SourcePreference::Mirror => vec![mirror, official.to_owned()],
        SourcePreference::Auto => vec![official.to_owned(), mirror],
        SourcePreference::Official => vec![official.to_owned()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn source_preferences_keep_official_fallback_and_both_library_mirrors() {
        use crate::io::net::http::resources::{DownloadOptions, SourceKind, SourcePreference};
        let bases = ApiEndpoints::default();
        let urls = |url: &str, preference: SourcePreference, official_fast: bool| {
            let options = DownloadOptions {
                file_source: preference,
                ..Default::default()
            };
            source_urls(&bases, url, SourceKind::File, &options, official_fast)
        };
        let official = "https://libraries.minecraft.net/org/test.jar";
        let mirror = "https://bmclapi2.bangbang93.com/maven/org/test.jar";
        let legacy = "https://bmclapi2.bangbang93.com/libraries/org/test.jar";
        assert_eq!(
            urls(official, SourcePreference::Mirror, true),
            vec![mirror, legacy, official]
        );
        assert_eq!(
            urls(mirror, SourcePreference::Official, false),
            vec![official, mirror, legacy]
        );
        assert_eq!(
            urls(official, SourcePreference::Auto, false),
            vec![mirror, legacy, official]
        );
        assert_eq!(
            urls(official, SourcePreference::Auto, true),
            vec![official, mirror, legacy]
        );
        let assets = urls(
            "https://bmclapi2.bangbang93.com/assets/ab/hash",
            SourcePreference::Official,
            true,
        );
        assert_eq!(
            assets[0],
            "https://resources.download.minecraft.net/ab/hash"
        );
        assert_eq!(
            urls(
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
            assert_eq!(urls(url, SourcePreference::Mirror, false), vec![url]);
        }
    }

    #[cfg(feature = "community")]
    #[test]
    fn community_sources_follow_their_own_preference() {
        use crate::io::net::http::resources::{DownloadOptions, SourceKind, SourcePreference};
        let bases = ApiEndpoints {
            meta_base: "https://private-mojang.test",
            ..Default::default()
        };
        let options = |community_source| DownloadOptions {
            community_source,
            ..Default::default()
        };
        let urls = |url: &str, preference| {
            source_urls(&bases, url, SourceKind::File, &options(preference), false)
        };
        let file = "https://edge.forgecdn.net/files/9453/343/a.jar";
        let mirrored = "https://mod.mcimirror.top/files/9453/343/a.jar";
        assert_eq!(urls(file, SourcePreference::Mirror), vec![mirrored, file]);
        assert_eq!(urls(file, SourcePreference::Auto), vec![file, mirrored]);
        assert_eq!(urls(file, SourcePreference::Official), vec![file]);
        for official in [
            "https://api.curseforge.com/v1/mods/1",
            "https://api.modrinth.com/v2/project/test",
        ] {
            assert_eq!(urls(official, SourcePreference::Official), vec![official]);
        }
        assert_eq!(
            urls(
                "https://api.curseforge.com/v1/mods/search?gameId=432",
                SourcePreference::Mirror
            )[0],
            "https://mod.mcimirror.top/curseforge/v1/mods/search?gameId=432"
        );
        assert_eq!(
            urls(
                "https://api.modrinth.com/v2/project/jei",
                SourcePreference::Mirror
            )[0],
            "https://mod.mcimirror.top/modrinth/v2/project/jei"
        );
        // mcim 不镜像该 CDN，换成镜像只会多一次 302。
        let mediafilez = "https://mediafilez.forgecdn.net/files/9453/343/a.jar";
        assert_eq!(urls(mediafilez, SourcePreference::Mirror), vec![mediafilez]);
        // 不相关的主机不受社区资源设置影响。
        let other = "https://example.org/files/a.jar";
        assert_eq!(urls(other, SourcePreference::Mirror), vec![other]);
    }

    #[test]
    fn mirrors_known_official_hosts_without_rewriting_third_party_origins() {
        let official = "https://authlib-injector.yushi.moe/artifact/test.jar?version=1";
        let mirror =
            "https://bmclapi2.bangbang93.com/mirrors/authlib-injector/artifact/test.jar?version=1";
        assert_eq!(authlib_sources(official), vec![official, mirror]);
        assert_eq!(authlib_sources(mirror), vec![official, mirror]);
        let unrelated = "https://authlib-injector.yushi.moe.evil.test/artifact/test.jar";
        assert_eq!(authlib_sources(unrelated), vec![unrelated]);

        let bases = ApiEndpoints {
            meta_base: BMCL,
            resources_base: "https://bmclapi2.bangbang93.com/assets",
            ..Default::default()
        };
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
            assert_eq!(
                mojang_url(url, bases.meta_base, bases.resources_base),
                expected
            );
        }
    }
}
