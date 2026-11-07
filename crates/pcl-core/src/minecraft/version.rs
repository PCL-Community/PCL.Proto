//! Mojang version metadata and the rules shared by installation and launch.
use std::collections::BTreeMap;

use anyhow::{Context, bail, ensure};
use serde::{Deserialize, Serialize};

pub const VERSION_MANIFEST_ENDPOINT: &str = "mc/game/version_manifest_v2.json";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McVersionManifest {
    pub latest: McLatestVersions,
    pub versions: Vec<McVersionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McLatestVersions {
    pub release: String,
    pub snapshot: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McVersionSummary {
    pub id: String,
    pub r#type: String,
    pub url: String,
    pub time: String,
    pub release_time: String,
    #[serde(default)]
    pub sha1: Option<String>,
}

#[allow(dead_code)]
impl McVersionSummary {
    pub fn category(&self) -> crate::minecraft::classifier::McVersionCategory {
        crate::minecraft::classifier::McVersionClassifier::classify_parts(&self.id, &self.r#type)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McVersion {
    pub id: String,
    pub r#type: String,
    pub downloads: McVersionDownloads,
    pub libraries: Vec<McLibrary>,
    pub assets: String,
    pub asset_index: McAssetIndex,
    pub main_class: String,
    pub logging: Option<McLogging>,
    #[serde(default)]
    pub arguments: Option<McVersionArguments>,
    #[serde(default)]
    pub minecraft_arguments: Option<String>,
    #[serde(default)]
    pub release_time: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McVersionArguments {
    #[serde(default)]
    pub game: Vec<McArgument>,
    #[serde(default)]
    pub jvm: Vec<McArgument>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McArgument {
    Plain(String),
    Conditional {
        rules: Vec<McRule>,
        value: McArgumentValue,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum McArgumentValue {
    One(String),
    Many(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McVersionDownloads {
    pub client: DownloadFile,
    pub server: Option<DownloadFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadFile {
    #[serde(default)]
    pub sha1: String,
    #[serde(default)]
    pub size: u64,
    #[serde(default)]
    pub url: String,
    pub path: Option<String>,
}

impl DownloadFile {
    pub fn validate(&self) -> anyhow::Result<()> {
        ensure!(valid_sha1(&self.sha1), "Invalid SHA-1: {}", self.sha1);
        let url = reqwest::Url::parse(&self.url)?;
        ensure!(url.scheme() == "https", "Download URL must use HTTPS");
        ensure!(
            url.username().is_empty() && url.password().is_none(),
            "Download URL contains credentials"
        );
        Ok(())
    }
}

pub fn valid_sha1(value: &str) -> bool {
    value.len() == 40 && value.bytes().all(|c| c.is_ascii_hexdigit())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McAssetIndex {
    pub id: String,
    #[serde(flatten)]
    pub download: DownloadFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McLogging {
    pub client: McLoggingClient,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McLoggingClient {
    pub argument: String,
    pub file: McLoggingFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McLoggingFile {
    pub id: String,
    #[serde(flatten)]
    pub download: DownloadFile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McRule {
    pub action: String,
    pub os: Option<McOsRule>,
    pub features: Option<BTreeMap<String, bool>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McOsRule {
    pub name: Option<String>,
    pub arch: Option<String>,
    pub version: Option<String>,
}

pub struct McRuleContext {
    pub os: String,
    pub arch: String,
    pub version: String,
    pub features: BTreeMap<String, bool>,
}

impl McRuleContext {
    pub fn current() -> Self {
        Self {
            os: match std::env::consts::OS {
                "macos" => "osx",
                os => os,
            }
            .into(),
            arch: std::env::consts::ARCH.into(),
            // MC 的 os.version 规则只用于少数旧版本；内核/系统版本即可满足。
            version: sysinfo::System::os_version().unwrap_or_default(),
            features: BTreeMap::new(),
        }
    }

    pub fn for_java(architecture: &crate::utils::platform::Architecture) -> anyhow::Result<Self> {
        use crate::utils::platform::Architecture;
        let arch = match architecture {
            Architecture::X86 => "x86",
            Architecture::X64 => "x86_64",
            Architecture::Arm64 => "aarch64",
            Architecture::FatFile | Architecture::Unknown => {
                bail!("Cannot determine the running Java architecture")
            }
        };
        let mut context = Self::current();
        context.arch = arch.into();
        Ok(context)
    }
}

fn pattern_matches(pattern: &Option<String>, value: &str) -> anyhow::Result<bool> {
    match pattern {
        Some(pattern) => Ok(regex::Regex::new(pattern)
            .with_context(|| format!("Invalid Minecraft rule pattern: {pattern}"))?
            .is_match(value)),
        None => Ok(true),
    }
}

/// Mojang rules start disallowed; the last matching rule wins.
pub fn rules_allow(rules: Option<&[McRule]>, context: &McRuleContext) -> anyhow::Result<bool> {
    let Some(rules) = rules else { return Ok(true) };
    let mut allowed = false;
    for rule in rules {
        ensure!(
            matches!(rule.action.as_str(), "allow" | "disallow"),
            "Invalid rule action: {}",
            rule.action
        );
        let os_matches = if let Some(os) = &rule.os {
            os.name.as_ref().is_none_or(|name| name == &context.os)
                && pattern_matches(&os.arch, &context.arch)?
                && pattern_matches(&os.version, &context.version)?
        } else {
            true
        };
        let features_match = rule.features.as_ref().is_none_or(|features| {
            features.iter().all(|(name, required)| {
                context.features.get(name).copied().unwrap_or(false) == *required
            })
        });
        if os_matches && features_match {
            allowed = rule.action == "allow";
        }
    }
    Ok(allowed)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McLibrary {
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
    #[serde(default)]
    pub size: Option<u64>,
    #[serde(default)]
    pub downloads: Option<McLibraryDownloads>,
    pub rules: Option<Vec<McRule>>,
    #[serde(default)]
    pub natives: BTreeMap<String, String>,
    pub extract: Option<McExtractRules>,
    #[serde(default)]
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McExtractRules {
    #[serde(default)]
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct McLibraryDownloads {
    pub artifact: Option<DownloadFile>,
    #[serde(default)]
    pub classifiers: BTreeMap<String, DownloadFile>,
}

impl McLibrary {
    pub fn classpath_path(
        &self,
        repo: &std::path::Path,
        instance: &std::path::Path,
    ) -> anyhow::Result<Option<std::path::PathBuf>> {
        if let Some(artifact) = self
            .downloads
            .as_ref()
            .and_then(|downloads| downloads.artifact.as_ref())
            && artifact.path.is_some()
        {
            return Ok(Some(library_path(repo, artifact)?));
        }
        if self
            .downloads
            .as_ref()
            .is_some_and(|downloads| downloads.artifact.is_none())
            || !self.natives.is_empty() && self.downloads.is_none()
        {
            return Ok(None);
        }
        let artifact = crate::minecraft::maven::MavenArtifact::parse(&self.name)
            .with_context(|| format!("Invalid library coordinate: {}", self.name))?;
        for part in artifact.group.split('.') {
            crate::io::files::validate_file_name(part)?;
        }
        for part in [
            Some(artifact.artifact.as_str()),
            Some(artifact.version.as_str()),
            Some(artifact.packaging.as_str()),
            artifact.classifier.as_deref(),
        ]
        .into_iter()
        .flatten()
        {
            crate::io::files::validate_file_name(part)?;
        }
        if self.hint.as_deref() == Some("local") {
            let classifier = artifact
                .classifier
                .as_deref()
                .map(|classifier| format!("-{classifier}"))
                .unwrap_or_default();
            let file = format!(
                "{}-{}{}.{}",
                artifact.artifact, artifact.version, classifier, artifact.packaging
            );
            return Ok(Some(instance.join("libraries").join(file)));
        }
        Ok(Some(crate::io::files::safe_join(
            &repo.join("libraries"),
            &artifact.relative_slash_path(),
        )?))
    }

    /// Identity used to de-duplicate inherited libraries.
    ///
    /// Version is intentionally omitted: a child version must replace the
    /// parent's artifact with the same group/artifact/classifier, matching CE.
    pub fn classpath_key(&self) -> anyhow::Result<String> {
        let artifact = crate::minecraft::maven::MavenArtifact::parse(&self.name)
            .with_context(|| format!("Invalid library coordinate: {}", self.name))?;
        Ok(format!(
            "{}:{}:{}@{}",
            artifact.group,
            artifact.artifact,
            artifact.classifier.as_deref().unwrap_or(""),
            artifact.packaging
        ))
    }

    pub fn rule_allow(&self, context: &McRuleContext) -> anyhow::Result<bool> {
        rules_allow(self.rules.as_deref(), context)
    }

    pub fn native_download(
        &self,
        context: &McRuleContext,
    ) -> anyhow::Result<Option<&DownloadFile>> {
        let Some(classifier) = self.natives.get(&context.os) else {
            return Ok(None);
        };
        let classifier =
            classifier.replace("${arch}", if context.arch == "x86" { "32" } else { "64" });
        match self
            .downloads
            .as_ref()
            .and_then(|downloads| downloads.classifiers.get(&classifier))
        {
            Some(info) => Ok(Some(info)),
            None => bail!("Missing native classifier {classifier} in {}", self.name),
        }
    }
}

pub fn library_path(
    repo: &std::path::Path,
    info: &DownloadFile,
) -> anyhow::Result<std::path::PathBuf> {
    crate::io::files::safe_join(
        &repo.join("libraries"),
        info.path
            .as_deref()
            .context("Missing library artifact path")?,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn context() -> McRuleContext {
        McRuleContext {
            os: "osx".into(),
            arch: "aarch64".into(),
            version: "14.0".into(),
            features: BTreeMap::new(),
        }
    }

    #[test]
    fn java_rule_context_uses_probed_architecture() {
        use crate::utils::platform::Architecture;
        for (arch, expected) in [
            (Architecture::X86, "x86"),
            (Architecture::X64, "x86_64"),
            (Architecture::Arm64, "aarch64"),
        ] {
            assert_eq!(McRuleContext::for_java(&arch).unwrap().arch, expected);
        }
        assert!(McRuleContext::for_java(&Architecture::Unknown).is_err());
        assert!(McRuleContext::for_java(&Architecture::FatFile).is_err());
    }

    #[test]
    fn ordered_rules_handle_unconditional_os_arch_version_and_features() {
        let parse = |value| serde_json::from_value::<Vec<McRule>>(value).unwrap();
        let rules = parse(json!([
            {"action":"allow"},
            {"action":"disallow","os":{"name":"osx"}},
            {"action":"allow","os":{"name":"osx","arch":"aarch64","version":"^14\\."}}
        ]));
        assert!(rules_allow(Some(&rules), &context()).unwrap());
        assert!(!rules_allow(Some(&rules[..2]), &context()).unwrap());
        assert!(!rules_allow(Some(&[]), &context()).unwrap());
        assert!(rules_allow(None, &context()).unwrap());
        let rules = parse(json!([{"action":"allow","features":{"is_demo_user":true}}]));
        assert!(!rules_allow(Some(&rules), &context()).unwrap());
        let rules = parse(json!([{"action":"allow","os":{"arch":"["}}]));
        assert!(rules_allow(Some(&rules), &context()).is_err());
    }

    #[test]
    fn classifier_only_libraries_resolve_native_downloads() {
        let library: McLibrary = serde_json::from_value(json!({
            "name":"org.lwjgl:lwjgl:2", "natives":{"osx":"natives-osx-${arch}"},
            "downloads":{"classifiers":{"natives-osx-64":{"sha1":"a".repeat(40),"size":1,"url":"https://example.org/native.jar"}}}
        })).unwrap();
        assert!(library.native_download(&context()).unwrap().is_some());
    }
}
