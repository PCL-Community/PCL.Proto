//! CE's vanilla installation stages, expressed as an async Rust service.
use crate::io::files::{check_sha1, safe_join, validate_file_name};
use crate::{
    io::download::{DownloadConfig, Downloader, StageProgress, TaskItemReport, TaskStatus},
    io::net::http::api_client::MinecraftApiClient,
    minecraft::version::{DownloadFile, McRuleContext, McVersion, library_path, valid_sha1},
};
use anyhow::{Context, ensure};
use futures_util::{StreamExt, TryStreamExt};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::ipc::Channel;
use tokio::sync::{Mutex, mpsc};

/// Atomically reserves the destination; never removes an existing instance.
struct InstallDirectory {
    path: PathBuf,
    committed: bool,
}

impl InstallDirectory {
    fn create(repo: &Path, name: &str) -> anyhow::Result<Self> {
        validate_file_name(name)?;
        let versions = repo.join("versions");
        fs::create_dir_all(&versions)?;
        let path = versions.join(name);
        fs::create_dir(&path).with_context(|| {
            format!(
                "Cannot install: destination already exists or is unavailable: {}",
                path.display()
            )
        })?;
        Ok(Self {
            path,
            committed: false,
        })
    }
}

impl Drop for InstallDirectory {
    fn drop(&mut self) {
        if !self.committed
            && let Err(error) = fs::remove_dir_all(&self.path)
        {
            log::warn!(
                "Cannot remove incomplete installation {}: {error}",
                self.path.display()
            );
        }
    }
}

#[derive(Deserialize)]
struct AssetIndex {
    objects: BTreeMap<String, AssetObject>,
    #[serde(default, rename = "virtual")]
    virtual_assets: bool,
    #[serde(default)]
    map_to_resources: bool,
}

#[derive(Deserialize)]
struct AssetObject {
    hash: String,
    size: u64,
}

struct NativeArchive {
    path: PathBuf,
    exclude: Vec<String>,
    lwjgl_core: bool,
}

struct InstallPlan {
    files: Vec<DownloadConfig>,
    natives: Vec<NativeArchive>,
    paths: HashMap<PathBuf, (String, u64)>,
    stages: BTreeMap<i32, Vec<u64>>,
}

impl InstallPlan {
    fn new() -> Self {
        Self {
            files: Vec::new(),
            natives: Vec::new(),
            paths: HashMap::new(),
            stages: [(1, vec![]), (2, vec![]), (3, vec![])].into(),
        }
    }

    fn add(&mut self, stage: i32, info: &DownloadFile, path: PathBuf) -> anyhow::Result<()> {
        info.validate()?;
        if let Some((sha1, size)) = self.paths.get(&path) {
            ensure!(
                sha1.eq_ignore_ascii_case(&info.sha1) && *size == info.size,
                "Conflicting download metadata for {}",
                path.display()
            );
            return Ok(());
        }
        self.paths
            .insert(path.clone(), (info.sha1.clone(), info.size));
        let sizes = self
            .stages
            .get_mut(&stage)
            .context("Unknown install stage")?;
        self.files.push(DownloadConfig {
            file_index: sizes.len(),
            task_item_id: stage,
            out_path: path,
            info: info.clone(),
        });
        sizes.push(info.size);
        Ok(())
    }

    fn libraries(
        &mut self,
        details: &McVersion,
        repo: &Path,
        context: &McRuleContext,
    ) -> anyhow::Result<()> {
        let mut native_keys = std::collections::HashSet::new();
        for library in &details.libraries {
            if !library.rule_allow(context)? {
                continue;
            }
            if let Some(info) = library
                .downloads
                .as_ref()
                .and_then(|downloads| downloads.artifact.as_ref())
            {
                self.add(2, info, library_path(repo, info)?)?;
            }
            if library.natives.contains_key(&context.os)
                && !native_keys.insert(library.classpath_key()?)
            {
                continue;
            }
            if library.downloads.is_none() && library.url.is_some() {
                continue;
            }
            if let Some(info) = library.native_download(context)? {
                let path = library_path(repo, info)?;
                self.add(2, info, path.clone())?;
                self.natives.push(NativeArchive {
                    path,
                    lwjgl_core: matches!(
                        library
                            .name
                            .split(':')
                            .take(2)
                            .collect::<Vec<_>>()
                            .as_slice(),
                        ["org.lwjgl", "lwjgl"] | ["org.lwjgl.lwjgl", "lwjgl-platform"]
                    ),
                    exclude: library
                        .extract
                        .as_ref()
                        .map(|rules| rules.exclude.clone())
                        .unwrap_or_default(),
                });
            }
        }
        if let Some(logging) = &details.logging {
            validate_file_name(&logging.client.file.id)?;
            self.add(
                2,
                &logging.client.file.download,
                repo.join("assets/log_configs")
                    .join(&logging.client.file.id),
            )?;
        }
        Ok(())
    }

    fn assets(
        &mut self,
        index: &AssetIndex,
        repo: &Path,
        resources_base: &str,
    ) -> anyhow::Result<()> {
        for (name, object) in &index.objects {
            // Both object hashes and legacy logical paths come from the network.
            safe_join(Path::new("assets"), name)?;
            ensure!(valid_sha1(&object.hash), "Invalid asset hash for {name}");
            let hash = object.hash.to_ascii_lowercase();
            let relative = format!("{}/{}", &hash[..2], hash);
            self.add(
                3,
                &DownloadFile {
                    sha1: hash,
                    size: object.size,
                    url: format!("{resources_base}/{relative}"),
                    path: None,
                },
                repo.join("assets/objects").join(relative),
            )?;
        }
        Ok(())
    }
}

async fn download_plan(
    plan: &mut InstallPlan,
    downloader: &Downloader,
    task_id: i32,
    on_event: &Channel<TaskItemReport>,
) -> anyhow::Result<()> {
    let mut stages: BTreeMap<_, _> = std::mem::take(&mut plan.stages)
        .into_iter()
        .map(|(id, sizes)| (id, StageProgress::new(id, sizes)))
        .collect();
    for stage in stages.values_mut() {
        let _ = on_event.send(stage.report(task_id));
    }
    let (tx, mut rx) = mpsc::channel::<crate::io::download::ProgressUpdate>(128);
    let download = async {
        let jobs = futures_util::stream::iter(std::mem::take(&mut plan.files).into_iter().map(
            |options| {
                let tx = tx.clone();
                async move {
                    let path = options.out_path.clone();
                    downloader
                        .start_download(options, tx)
                        .await
                        .with_context(|| format!("Cannot download {}", path.display()))
                }
            },
        ))
        // The shared quota controls requests, including live connection-limit changes.
        .buffer_unordered(256)
        .try_collect::<Vec<_>>()
        .await;
        drop(tx);
        jobs
    };
    let monitor = async {
        while let Some(update) = rx.recv().await {
            if let Some(stage) = stages.get_mut(&update.item_id) {
                stage.update(update.file_index, update.progress);
                if stage.should_report() {
                    let _ = on_event.send(stage.report(task_id));
                }
            }
        }
    };
    let (result, ()) = tokio::join!(download, monitor);
    result?;
    Ok(())
}

async fn add_maven_libraries(
    plan: &mut InstallPlan,
    details: &McVersion,
    repo: &Path,
    instance: &Path,
    context: &McRuleContext,
    downloader: &Downloader,
) -> anyhow::Result<()> {
    use crate::io::download::resources::DOWNLOAD_RESOURCES;
    let mut seen = std::collections::HashSet::new();
    for library in &details.libraries {
        if !library.rule_allow(context)?
            || (!library.natives.is_empty() && !library.natives.contains_key(&context.os))
            || !seen.insert(library.classpath_key()?)
        {
            continue;
        }
        if library.downloads.is_some() || library.hint.as_deref() == Some("local") {
            continue;
        }
        let Some(base) = library.url.as_deref() else {
            continue;
        };
        let native = if library.natives.is_empty() {
            None
        } else {
            let Some(classifier) = library.natives.get(&context.os) else {
                continue;
            };
            Some(classifier.replace("${arch}", if context.arch == "x86" { "32" } else { "64" }))
        };
        let mut coordinate = library.clone();
        if let Some(classifier) = &native {
            let (name, extension) = coordinate
                .name
                .split_once('@')
                .unwrap_or((&coordinate.name, "jar"));
            ensure!(
                name.split(':').count() == 3,
                "Invalid native Maven coordinate"
            );
            coordinate.name = format!("{name}:{classifier}@{extension}");
            coordinate.natives.clear();
        }
        let path = coordinate
            .classpath_path(repo, instance)?
            .context("Missing Maven library path")?;
        let relative = path.strip_prefix(repo.join("libraries"))?;
        let mut url = reqwest::Url::parse(base).context("Invalid Maven repository URL")?;
        ensure!(
            url.scheme() == "https"
                && url.host_str().is_some()
                && url.username().is_empty()
                && url.password().is_none()
                && url.query().is_none()
                && url.fragment().is_none(),
            "Invalid Maven repository URL"
        );
        if !url.path().ends_with('/') {
            let path = format!("{}/", url.path());
            url.set_path(&path);
        }
        {
            let mut segments = url
                .path_segments_mut()
                .map_err(|_| anyhow::anyhow!("Invalid Maven repository URL"))?;
            segments.pop_if_empty();
            for component in relative.components() {
                segments.push(
                    component
                        .as_os_str()
                        .to_str()
                        .context("Invalid Maven path")?,
                );
            }
        }
        let mut checksum_url = url.clone();
        checksum_url.set_path(&format!("{}.sha1", url.path()));
        let resources = &*DOWNLOAD_RESOURCES;
        let sha1 = if let Some(sha1) = library.sha1.as_ref().filter(|_| native.is_none()) {
            sha1.clone()
        } else {
            let _lease = resources.acquire().await;
            let response = maven_metadata_request(downloader, checksum_url.as_str()).await?;
            let mut stream = response.bytes_stream();
            let mut bytes = Vec::new();
            while let Some(chunk) = stream.next().await {
                let chunk = chunk?;
                ensure!(
                    bytes.len() + chunk.len() <= 1024,
                    "Maven checksum response is too large"
                );
                resources.record_bytes(chunk.len());
                resources.throttle(chunk.len()).await;
                bytes.extend_from_slice(&chunk);
            }
            String::from_utf8(bytes)?
                .split_whitespace()
                .next()
                .unwrap_or("")
                .to_owned()
        };
        ensure!(
            valid_sha1(&sha1),
            "Invalid Maven checksum for {}",
            library.name
        );
        let size = if let Some(size) = library.size.filter(|_| native.is_none()) {
            size
        } else {
            let _lease = resources.acquire().await;
            let response = maven_metadata_request(downloader, url.as_str()).await?;
            response
                .content_length()
                .context("Maven library has no content length")?
        };
        ensure!(
            size > 0 && size <= 256 * 1024 * 1024,
            "Invalid Maven library size"
        );
        plan.add(
            2,
            &DownloadFile {
                sha1,
                size,
                url: url.to_string(),
                path: None,
            },
            path.clone(),
        )?;
        if native.is_some() {
            plan.natives.push(NativeArchive {
                path,
                lwjgl_core: matches!(
                    library
                        .name
                        .split(':')
                        .take(2)
                        .collect::<Vec<_>>()
                        .as_slice(),
                    ["org.lwjgl", "lwjgl"] | ["org.lwjgl.lwjgl", "lwjgl-platform"]
                ),
                exclude: library
                    .extract
                    .as_ref()
                    .map(|rules| rules.exclude.clone())
                    .unwrap_or_default(),
            });
        }
    }
    Ok(())
}

async fn maven_metadata_request(
    downloader: &Downloader,
    url: &str,
) -> anyhow::Result<reqwest::Response> {
    use crate::io::download::{resources::DOWNLOAD_RESOURCES, send_request};
    let mut last_error = None;
    for source in downloader.source_urls(url) {
        match send_request(
            DOWNLOAD_RESOURCES.settings().http_mode,
            false,
            &source,
            None,
            None,
        )
        .await
        .and_then(reqwest::Response::error_for_status)
        {
            Ok(response) => return Ok(response),
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.context("No Maven source available")?.into())
}

fn extract_natives(archives: &[NativeArchive], destination: &Path) -> anyhow::Result<()> {
    fs::create_dir_all(destination)?;
    for native in archives {
        let mut archive = zip::ZipArchive::new(fs::File::open(&native.path)?)?;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            let name = entry.name()?.into_owned();
            if entry.is_dir()
                || name.starts_with("META-INF/")
                || native
                    .exclude
                    .iter()
                    .any(|exclude| name.starts_with(exclude))
            {
                continue;
            }
            ensure!(
                entry
                    .unix_mode()
                    .is_none_or(|mode| mode & 0o170000 != 0o120000),
                "Native archive contains a symbolic link"
            );
            let path = safe_join(destination, &name)?;
            fs::create_dir_all(path.parent().context("Missing native parent")?)?;
            let mut file = fs::File::create(&path)?;
            std::io::copy(&mut entry, &mut file)?;
        }
    }
    Ok(())
}

fn native_binary_matches(bytes: &[u8], os: &str, arch: &str) -> anyhow::Result<bool> {
    use goblin::{Object, elf::header, mach, pe};
    let expected_mach = match arch {
        "x86" => mach::constants::cputype::CPU_TYPE_X86,
        "x86_64" => mach::constants::cputype::CPU_TYPE_X86_64,
        "aarch64" => mach::constants::cputype::CPU_TYPE_ARM64,
        _ => anyhow::bail!("Unknown Java architecture: {arch}"),
    };
    Ok(match (os, Object::parse(bytes)?) {
        ("osx", Object::Mach(mach::Mach::Binary(binary))) => binary.header.cputype == expected_mach,
        ("osx", Object::Mach(mach::Mach::Fat(binary))) => {
            binary.arches()?.iter().any(|a| a.cputype == expected_mach)
        }
        ("linux", Object::Elf(binary)) => {
            binary.header.e_machine
                == match arch {
                    "x86" => header::EM_386,
                    "x86_64" => header::EM_X86_64,
                    _ => header::EM_AARCH64,
                }
        }
        ("windows", Object::PE(binary)) => {
            binary.header.coff_header.machine
                == match arch {
                    "x86" => pe::header::COFF_MACHINE_X86,
                    "x86_64" => pe::header::COFF_MACHINE_X86_64,
                    _ => pe::header::COFF_MACHINE_ARM64,
                }
        }
        _ => false,
    })
}

fn validate_lwjgl_natives(
    archives: &[NativeArchive],
    context: &McRuleContext,
) -> anyhow::Result<()> {
    use std::io::Read;
    // shortcut: only legacy LWJGL core is checked; extend to loader-specific native contracts when supported.
    for native in archives.iter().filter(|native| native.lwjgl_core) {
        let mut archive = zip::ZipArchive::new(fs::File::open(&native.path)?)?;
        let mut matched = false;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            let entry_name = entry.name()?.into_owned();
            let name = entry_name
                .rsplit('/')
                .next()
                .unwrap_or("")
                .to_ascii_lowercase();
            if entry.is_dir()
                || entry_name.starts_with("META-INF/")
                || native
                    .exclude
                    .iter()
                    .any(|exclude| entry_name.starts_with(exclude))
                || !name.contains("lwjgl")
                || ![".dll", ".so", ".dylib", ".jnilib"]
                    .iter()
                    .any(|suffix| name.ends_with(suffix))
            {
                continue;
            }
            ensure!(
                entry.size() <= 64 * 1024 * 1024,
                "LWJGL native is too large: {name}"
            );
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes)?;
            if native_binary_matches(&bytes, &context.os, &context.arch).with_context(|| {
                format!("Invalid LWJGL native {name} in {}", native.path.display())
            })? {
                matched = true;
            }
        }
        ensure!(
            matched,
            "LWJGL native archive {} does not support Java architecture {} on {}; select a compatible Java or install compatible natives",
            native.path.display(),
            context.arch,
            context.os
        );
    }
    Ok(())
}

pub struct LaunchNatives(PathBuf);

impl LaunchNatives {
    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for LaunchNatives {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_dir_all(&self.0) {
            log::warn!("Cannot remove launch natives {}: {error}", self.0.display());
        }
    }
}

pub async fn prepare_launch_libraries(
    details: &McVersion,
    repo: &Path,
    instance: &Path,
    context: &McRuleContext,
    downloader: &Downloader,
    verify: bool,
) -> anyhow::Result<LaunchNatives> {
    let mut plan = InstallPlan::new();
    plan.libraries(details, repo, context)?;
    add_maven_libraries(&mut plan, details, repo, instance, context, downloader).await?;
    futures_util::stream::iter(std::mem::take(&mut plan.files))
        .map(|file| {
            let downloader = downloader.clone();
            async move {
                let result = if verify {
                    downloader.download(&file.info, &file.out_path).await
                } else {
                    downloader
                        .download_unverified(&file.info, &file.out_path)
                        .await
                };
                result.with_context(|| format!("Cannot prepare {}", file.out_path.display()))
            }
        })
        .buffer_unordered(64)
        .try_collect::<Vec<_>>()
        .await?;
    let path = std::env::temp_dir().join(format!("pcl-natives-{}", uuid::Uuid::new_v4()));
    fs::create_dir(&path)?;
    let natives = LaunchNatives(path);
    let context = McRuleContext {
        os: context.os.clone(),
        arch: context.arch.clone(),
        version: context.version.clone(),
        features: context.features.clone(),
    };
    tokio::task::spawn_blocking(move || {
        validate_lwjgl_natives(&plan.natives, &context)?;
        extract_natives(&plan.natives, natives.path())?;
        Ok::<_, anyhow::Error>(natives)
    })
    .await?
}

pub async fn prepare_legacy_assets(
    details: &McVersion,
    repo: &Path,
    game_directory: &Path,
    downloader: &Downloader,
    verify: bool,
) -> anyhow::Result<()> {
    validate_file_name(&details.asset_index.id)?;
    let index_path = repo
        .join("assets/indexes")
        .join(format!("{}.json", details.asset_index.id));
    if details.asset_index.download.validate().is_ok() {
        if verify {
            downloader
                .download(&details.asset_index.download, &index_path)
                .await?;
        } else {
            downloader
                .download_unverified(&details.asset_index.download, &index_path)
                .await?;
        }
    } else {
        ensure!(
            index_path.is_file(),
            "Asset index has no valid download metadata and is missing: {}",
            index_path.display()
        );
    }
    let index: AssetIndex = serde_json::from_slice(&tokio::fs::read(index_path).await?)
        .context("Invalid asset index")?;
    if !index.map_to_resources && !index.virtual_assets {
        return Ok(());
    }
    let mut plan = InstallPlan::new();
    plan.assets(&index, repo, "https://resources.download.minecraft.net")?;
    futures_util::stream::iter(plan.files)
        .map(|file| async move {
            if verify {
                downloader.download(&file.info, &file.out_path).await
            } else {
                downloader
                    .download_unverified(&file.info, &file.out_path)
                    .await
            }
        })
        .buffer_unordered(16)
        .try_collect::<Vec<_>>()
        .await?;
    let repo = repo.to_owned();
    let game_directory = game_directory.to_owned();
    let index_id = details.asset_index.id.clone();
    tokio::task::spawn_blocking(move || {
        materialize_assets(&index, &repo, &game_directory, &index_id, verify)
    })
    .await??;
    Ok(())
}

fn materialize_assets(
    index: &AssetIndex,
    repo: &Path,
    instance: &Path,
    index_id: &str,
    verify: bool,
) -> anyhow::Result<()> {
    let mut roots = Vec::new();
    if index.virtual_assets {
        roots.push(repo.join("assets/virtual").join(index_id));
    }
    if index.map_to_resources {
        roots.push(instance.join("resources"));
    }
    for root in roots {
        fs::create_dir_all(&root)?;
        ensure!(
            !fs::symlink_metadata(&root)?.file_type().is_symlink(),
            "Asset root is a symlink: {}",
            root.display()
        );
        for (name, object) in &index.objects {
            ensure!(valid_sha1(&object.hash), "Invalid asset hash for {name}");
            let hash = object.hash.to_ascii_lowercase();
            let source = repo.join("assets/objects").join(&hash[..2]).join(&hash);
            let target = safe_join(&root, name)?;
            let parent = target.parent().context("Missing asset parent")?;
            let mut directory = root.clone();
            for component in parent.strip_prefix(&root)?.components() {
                directory.push(component);
                match fs::create_dir(&directory) {
                    Ok(()) => {}
                    Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                    Err(error) => return Err(error.into()),
                }
                ensure!(
                    fs::symlink_metadata(&directory)?.file_type().is_dir(),
                    "Invalid asset directory: {}",
                    directory.display()
                );
            }
            match fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&target)
            {
                Ok(mut output) => {
                    let result = (|| -> anyhow::Result<()> {
                        let mut input = fs::File::open(&source)?;
                        std::io::copy(&mut input, &mut output)?;
                        Ok(())
                    })();
                    if result.is_err() {
                        drop(output);
                        let _ = fs::remove_file(&target);
                    }
                    result?;
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                    let metadata = fs::symlink_metadata(&target)?;
                    ensure!(
                        metadata.file_type().is_file()
                            && (if verify {
                                metadata.len() == object.size && check_sha1(&target, &hash)?
                            } else {
                                true
                            }),
                        "Asset conflicts with existing file: {}",
                        target.display()
                    );
                }
                Err(error) => return Err(error.into()),
            }
        }
    }
    Ok(())
}

fn merge_fabric_profile(
    metadata: &mut serde_json::Value,
    profile: &serde_json::Value,
) -> anyhow::Result<()> {
    let loader_libraries = profile["libraries"]
        .as_array()
        .context("Invalid Fabric libraries")?;
    let vanilla_libraries = metadata["libraries"]
        .as_array()
        .context("Invalid Minecraft libraries")?;
    let mut libraries = loader_libraries.clone();
    libraries.extend(vanilla_libraries.iter().cloned());
    let mut arguments = metadata
        .get("arguments")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));
    ensure!(arguments.is_object(), "Invalid Minecraft arguments");
    for key in ["jvm", "game"] {
        let loader = profile.pointer(&format!("/arguments/{key}"));
        let vanilla = arguments.get(key);
        let mut merged = match loader {
            Some(value) => value
                .as_array()
                .context("Invalid Fabric arguments")?
                .clone(),
            None => Vec::new(),
        };
        if let Some(value) = vanilla {
            merged.extend(
                value
                    .as_array()
                    .context("Invalid Minecraft arguments")?
                    .iter()
                    .cloned(),
            );
        } else if key == "game" && !merged.is_empty() {
            if let Some(legacy) = metadata.get("minecraftArguments") {
                let legacy = legacy.as_str().context("Invalid legacy game arguments")?;
                merged.extend(
                    shlex::split(legacy)
                        .context("Invalid legacy game arguments")?
                        .into_iter()
                        .map(serde_json::Value::String),
                );
            }
        }
        arguments[key] = merged.into();
    }
    metadata["libraries"] = libraries.into();
    metadata["arguments"] = arguments;
    metadata["mainClass"] = profile["mainClass"].clone();
    Ok(())
}

#[derive(serde::Deserialize)]
pub struct FabricInstall {
    family: crate::io::net::http::api_client::FabricFamily,
    version: String,
}

async fn install_version(
    client: &MinecraftApiClient,
    repo: &Path,
    version_id: &str,
    name: &str,
    fabric: Option<&FabricInstall>,
    task_id: i32,
    on_event: &Channel<TaskItemReport>,
) -> anyhow::Result<()> {
    validate_file_name(version_id)?;
    ensure!(
        repo.is_absolute() && repo.is_dir(),
        "Minecraft folder must be an existing absolute directory"
    );
    let repo = repo.canonicalize()?;
    let mut directory = InstallDirectory::create(&repo, name)?;
    log::info!(
        "Installing Minecraft {version_id} as {name} in {}",
        repo.display()
    );
    let _ = on_event.send(TaskItemReport {
        task_id,
        item_id: 0,
        files_remaining: 1,
        progress: 0.0,
        status: TaskStatus::Running,
        speed: None,
    });
    let mut metadata = client.get_version_metadata(version_id).await?;
    if let Some(loader) = fabric {
        let profile = client
            .get_fabric_profile(version_id, &loader.version, loader.family)
            .await?;
        merge_fabric_profile(&mut metadata, &profile)?;
    }
    let details: McVersion =
        serde_json::from_value(metadata.clone()).context("Invalid vanilla version metadata")?;
    validate_file_name(&details.asset_index.id)?;
    details.asset_index.download.validate()?;
    let bases = client.api_bases_async().await;
    let downloader = Downloader::with_bases(bases.clone());
    let index_path = repo
        .join("assets/indexes")
        .join(format!("{}.json", details.asset_index.id));
    downloader
        .download(&details.asset_index.download, &index_path)
        .await?;
    let index: AssetIndex = serde_json::from_slice(&tokio::fs::read(&index_path).await?)
        .context("Invalid asset index")?;
    let mut plan = InstallPlan::new();
    plan.add(
        1,
        &details.downloads.client,
        directory.path.join(format!("{name}.jar")),
    )?;
    plan.libraries(&details, &repo, &McRuleContext::current())?;
    add_maven_libraries(
        &mut plan,
        &details,
        &repo,
        &directory.path,
        &McRuleContext::current(),
        &downloader,
    )
    .await?;
    plan.assets(&index, &repo, "https://resources.download.minecraft.net")?;
    let _ = on_event.send(TaskItemReport {
        task_id,
        item_id: 0,
        files_remaining: 0,
        progress: 1.0,
        status: TaskStatus::Completed,
        speed: None,
    });
    download_plan(&mut plan, &downloader, task_id, on_event).await?;
    let instance_path = directory.path.clone();
    let index_id = details.asset_index.id;
    tokio::task::spawn_blocking(move || {
        extract_natives(&plan.natives, &instance_path.join("natives"))?;
        materialize_assets(&index, &repo, &instance_path, &index_id, true)
    })
    .await??;
    // CE preserves the source version separately when the instance has a custom name.
    metadata["id"] = name.into();
    metadata["clientVersion"] = version_id.into();
    let temporary_json = directory.path.join(".pcl-version.json.part");
    tokio::fs::write(&temporary_json, serde_json::to_vec_pretty(&metadata)?).await?;
    // The instance becomes discoverable only after every install stage succeeds.
    tokio::fs::rename(temporary_json, directory.path.join(format!("{name}.json"))).await?;
    directory.committed = true;
    log::info!(
        "Installed Minecraft {version_id}: {}",
        directory.path.display()
    );
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_minecraft_version(
    state: tauri::State<'_, Arc<Mutex<crate::app::config::AppState>>>,
    client: tauri::State<'_, &MinecraftApiClient>,
    on_event: Channel<TaskItemReport>,
    version_id: String,
    task_id: i32,
    instance_name: String,
    fabric: Option<FabricInstall>,
) -> Result<InstallResult, String> {
    let repo = state.lock().await.active_repo_path.clone();
    install_version(
        &client,
        &repo,
        &version_id,
        &instance_name,
        fabric.as_ref(),
        task_id,
        &on_event,
    )
    .await
    .map_err(|error| format!("{error:#}"))?;
    let should_select = state
        .lock()
        .await
        .download_settings
        .as_ref()
        .and_then(|settings| settings.auto_select_instance)
        .unwrap_or(false);
    if !should_select {
        return Ok(InstallResult {
            selected: false,
            selection_error: None,
        });
    }
    let selection = tauri::async_runtime::spawn_blocking(move || {
        let repository = crate::minecraft::instance::folder::McFolder::new("", repo.clone());
        let instance = crate::minecraft::instance::McInstance::from_version_folder(
            &repo.join("versions").join(instance_name),
            &repository,
        )
        .map_err(|error| format!("{error:?}"))?;
        crate::app::config::ConfigManager::instance().select_installed_instance(&repo, instance)
    })
    .await
    .map_err(|error| error.to_string())
    .and_then(|result| result);
    Ok(match selection {
        Ok(selected) => InstallResult {
            selected,
            selection_error: None,
        },
        Err(error) => InstallResult {
            selected: false,
            selection_error: Some(error),
        },
    })
}

#[derive(serde::Serialize)]
pub struct InstallResult {
    selected: bool,
    selection_error: Option<String>,
}

#[cfg(test)]
mod tests;
