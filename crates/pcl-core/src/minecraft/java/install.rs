//! Install Mojang runtimes without archives; share verified transfers with Minecraft installation.
use super::{JavaEntry, JavaSource, requirements::JavaRequirements};
use crate::{
    io::download::Downloader,
    io::files::{safe_join, validate_file_name},
    minecraft::version::DownloadFile,
};
use anyhow::{Context, ensure};
use futures_util::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

const INDEX: &str = "https://piston-meta.mojang.com/v1/products/java-runtime/2ec0cc96c44e5a76b9c8b7c39df7210883d12871/all.json";
#[derive(Deserialize)]
struct RuntimeRelease {
    manifest: DownloadFile,
    version: RuntimeVersion,
}
#[derive(Deserialize)]
struct RuntimeVersion {
    name: String,
}
#[derive(Deserialize)]
struct RuntimeManifest {
    files: BTreeMap<String, RuntimeFile>,
}
#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum RuntimeFile {
    Directory,
    File {
        downloads: RawDownload,
        #[serde(default)]
        executable: bool,
    },
    Link {
        target: String,
    },
}
#[derive(Deserialize)]
struct RawDownload {
    raw: DownloadFile,
}
#[derive(Serialize, Clone)]
pub struct JavaInstallProgress {
    pub completed: usize,
    pub total: usize,
}

struct InstallDirectory {
    path: PathBuf,
    committed: bool,
}
impl Drop for InstallDirectory {
    fn drop(&mut self) {
        if !self.committed {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }
}

pub(super) fn platform_key(os: &str, arch: &str) -> Option<&'static str> {
    match (os, arch) {
        ("windows", "x86_64") => Some("windows-x64"),
        ("windows", "x86" | "i686") => Some("windows-x86"),
        ("windows", "aarch64") => Some("windows-arm64"),
        ("macos", "x86_64") => Some("mac-os"),
        ("macos", "aarch64") => Some("mac-os-arm64"),
        ("linux", "x86_64") => Some("linux"),
        ("linux", "x86" | "i686") => Some("linux-i386"),
        _ => None,
    }
}

pub async fn install(
    requirements: JavaRequirements,
    runtime_dir: PathBuf,
    downloader: Downloader,
    progress: impl Fn(JavaInstallProgress) + Clone + Send + Sync + 'static,
) -> anyhow::Result<JavaEntry> {
    let major = requirements
        .download_major()
        .context("This Java version/update range requires manual installation")?;
    let platform = platform_key(
        std::env::consts::OS,
        crate::utils::platform::host_architecture(),
    )
    .context("Mojang does not provide a runtime for this OS/architecture; install Java manually")?;
    let client = downloader.http_client();
    let index: BTreeMap<String, BTreeMap<String, Vec<RuntimeRelease>>> =
        fetch_index(&client, &downloader).await?;
    let components = index
        .get(platform)
        .context("No Mojang runtimes available for this platform")?;
    let matches = |release: &&RuntimeRelease| {
        // Legacy releases use 8u202 rather than Java's 1.8.0_202 version spelling.
        release
            .version
            .name
            .split(['.', 'u', '_', '-'])
            .next()
            .and_then(|v| v.parse::<u32>().ok())
            == Some(major)
    };
    let exact = requirements
        .component
        .as_ref()
        .and_then(|c| components.get_key_value(c))
        .and_then(|(c, rs)| rs.iter().find(matches).map(|r| (c, r)));
    let (component, release) = exact.or_else(|| components.iter().find_map(|(c, rs)| rs.iter().find(matches).map(|r| (c, r)))).context("Mojang does not provide the required Java version for this OS/architecture; install it manually")?;
    validate_file_name(component)?;
    release.manifest.validate()?;
    tokio::fs::create_dir_all(&runtime_dir).await?;
    let unique = uuid::Uuid::new_v4();
    let staging = runtime_dir.join(format!(".install-{unique}"));
    std::fs::create_dir(&staging)?;
    let mut directory = InstallDirectory {
        path: staging.clone(),
        committed: false,
    };
    let manifest_file = staging.join("runtime-manifest.json");
    downloader
        .download(&release.manifest, &manifest_file)
        .await?;
    let manifest: RuntimeManifest =
        serde_json::from_slice(&tokio::fs::read(&manifest_file).await?)?;
    let mut downloads = Vec::new();
    let mut links = Vec::new();
    for (name, file) in &manifest.files {
        let path = safe_join(&staging, name)?;
        match file {
            RuntimeFile::Directory => std::fs::create_dir_all(path)?,
            RuntimeFile::File {
                downloads: data,
                executable,
            } => {
                data.raw.validate()?;
                downloads.push((path, data.raw.clone(), *executable));
            }
            RuntimeFile::Link { target } => {
                links.push((path, target.clone()));
            }
        }
    }
    let total = downloads.len();
    ensure!(total > 0, "Empty Java runtime manifest");
    let completed = Arc::new(AtomicUsize::new(0));
    progress(JavaInstallProgress {
        completed: 0,
        total,
    });
    futures_util::stream::iter(downloads.into_iter().map(|(path, info, executable)| {
        let downloader = downloader.clone();
        let completed = completed.clone();
        let progress = progress.clone();
        async move {
            downloader.download(&info, &path).await?;
            #[cfg(unix)]
            if executable {
                use std::os::unix::fs::PermissionsExt;
                tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).await?;
            }
            #[cfg(not(unix))]
            let _ = executable;
            let count = completed.fetch_add(1, Ordering::Relaxed) + 1;
            progress(JavaInstallProgress {
                completed: count,
                total,
            });
            Ok::<_, anyhow::Error>(())
        }
    }))
    .buffer_unordered(16)
    .try_collect::<Vec<_>>()
    .await?;
    // Materialize links after downloads. Canonical targets must remain inside our fresh staging directory.
    while !links.is_empty() {
        let before = links.len();
        let mut remaining = Vec::new();
        for (path, target) in links {
            let resolved = link_target(&staging, &path, &target)?;
            if !resolved.exists() {
                remaining.push((path, target));
                continue;
            }
            ensure!(
                resolved
                    .canonicalize()?
                    .starts_with(staging.canonicalize()?),
                "Java link escapes installation directory"
            );
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            #[cfg(unix)]
            std::os::unix::fs::symlink(target, &path)?;
            // Windows file copies avoid requiring Developer Mode or elevated symlink privileges.
            #[cfg(windows)]
            {
                ensure!(
                    resolved.is_file(),
                    "Unsupported directory link in Java runtime"
                );
                tokio::fs::copy(resolved, path).await?;
            }
        }
        ensure!(
            remaining.len() < before,
            "Unresolved or cyclic links in Java runtime"
        );
        links = remaining;
    }
    let executable = manifest
        .files
        .keys()
        .find(|name| {
            name.ends_with(&format!("/bin/{}", super::java_binary()))
                || name.as_str() == format!("bin/{}", super::java_binary())
        })
        .context("Java executable missing from runtime manifest")?;
    let relative = PathBuf::from(executable);
    let destination = runtime_dir.join(format!("{component}-{platform}-{unique}"));
    std::fs::rename(&staging, &destination)?;
    directory.path = destination.clone();
    let path = destination.join(relative);
    let mut runtime = tokio::task::spawn_blocking(move || JavaEntry::probe(&path)).await??;
    ensure!(
        requirements.accepts(runtime.parsed_version()),
        "Downloaded Java {} does not satisfy {}",
        runtime.version,
        requirements.description()
    );
    ensure!(
        !matches!(
            runtime.installation.compatibility,
            crate::utils::platform::Compatibility::No
                | crate::utils::platform::Compatibility::Unknown
        ),
        "Downloaded Java has an incompatible architecture"
    );
    // Mojang ships complete game runtimes without javac; these are explicitly installed, trusted candidates.
    runtime.is_enabled = true;
    runtime.source = JavaSource::ManualAdded;
    directory.committed = true;
    Ok(runtime)
}

async fn fetch_index(
    client: &reqwest::Client,
    downloader: &Downloader,
) -> anyhow::Result<BTreeMap<String, BTreeMap<String, Vec<RuntimeRelease>>>> {
    let mut last = None;
    let settings = downloader.resources().settings();
    for url in downloader.bases.download_urls(
        INDEX,
        crate::io::download::resources::SourceKind::VersionMetadata,
        &settings,
        false,
    ) {
        let result = async {
            let mut response = client.get(url).send().await?.error_for_status()?;
            let mut bytes = Vec::new();
            while let Some(chunk) = response.chunk().await? {
                ensure!(
                    bytes.len() + chunk.len() <= 4 * 1024 * 1024,
                    "Oversized Java index"
                );
                bytes.extend_from_slice(&chunk);
            }
            Ok::<_, anyhow::Error>(serde_json::from_slice(&bytes)?)
        }
        .await;
        match result {
            Ok(index) => return Ok(index),
            Err(e) => last = Some(e),
        }
    }
    Err(last.context("No Java metadata source available")?)
}

pub(super) fn link_target(root: &Path, path: &Path, target: &str) -> anyhow::Result<PathBuf> {
    ensure!(
        !target.contains('\\') && !target.contains(':'),
        "Invalid Java link target"
    );
    let target = Path::new(target);
    ensure!(!target.is_absolute(), "Absolute Java link target");
    let mut resolved = path
        .parent()
        .context("Missing Java link parent")?
        .to_owned();
    for part in target.components() {
        match part {
            std::path::Component::Normal(name) => resolved.push(name),
            std::path::Component::CurDir => (),
            std::path::Component::ParentDir => {
                ensure!(
                    resolved != root && resolved.pop(),
                    "Java link escapes installation directory"
                );
            }
            _ => anyhow::bail!("Invalid Java link target"),
        }
    }
    ensure!(
        resolved.starts_with(root),
        "Java link escapes installation directory"
    );
    Ok(resolved)
}
