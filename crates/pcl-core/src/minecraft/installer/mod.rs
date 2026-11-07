//! CE's vanilla installation stages, expressed as an async Rust service.
pub mod forgelike;
pub mod modpack;
use crate::events::EventSink;
use crate::io::files::{check_sha1, safe_join, validate_file_name};
use crate::{
    io::download::{DownloadConfig, Downloader, StageProgress, TaskItemReport, TaskStatus},
    io::net::http::api_client::MinecraftApiClient,
    minecraft::version::{DownloadFile, McRuleContext, McVersion, library_path, valid_sha1},
};
use anyhow::{Context, ensure};
use futures_util::{StreamExt, TryStreamExt};
use serde::Deserialize;
use sha1::Digest;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
    io::Read,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::sync::mpsc;

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
        stage: i32,
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
                let path = library_path(repo, info)?;
                if info.url.is_empty() {
                    ensure!(
                        valid_sha1(&info.sha1)
                            && fs::metadata(&path).is_ok_and(|file| file.is_file()
                                && (info.size == 0 || file.len() == info.size))
                            && check_sha1(&path, &info.sha1)?,
                        "Missing or invalid local library: {}",
                        path.display()
                    );
                } else {
                    self.add(stage, info, path)?;
                }
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
                self.add(stage, info, path.clone())?;
                self.natives.push(NativeArchive {
                    path,
                    lwjgl_core: matches!(
                        library
                            .name
                            .split(':')
                            .take(2)
                            .collect::<Vec<_>>()
                            .as_slice(),
                        ["org.lwjgl", "lwjgl"]
                            | ["org.lwjgl.lwjgl", "lwjgl-platform"]
                            | ["org.glavo.hmcl", "lwjgl2-natives"]
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
                stage,
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
    on_event: &EventSink<TaskItemReport>,
) -> anyhow::Result<()> {
    let mut stages: BTreeMap<_, _> = std::mem::take(&mut plan.stages)
        .into_iter()
        .map(|(id, sizes)| (id, StageProgress::new(id, sizes)))
        .collect();
    for stage in stages.values_mut() {
        on_event(stage.report(task_id));
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
                    on_event(stage.report(task_id));
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
    stage: i32,
) -> anyhow::Result<()> {
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
        let resources = downloader.resources();
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
            stage,
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
    let mut last_error = None;
    for source in downloader.source_urls(url) {
        match downloader
            .send_request(
                downloader.resources().settings().http_mode,
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
            // 头里声明的 size 不可信，按实际解压字节限 512 MiB。
            crate::io::files::copy_entry_bounded(&mut entry, &mut file, 512 * 1024 * 1024)?;
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

fn natives_digest(root: &Path) -> anyhow::Result<String> {
    ensure!(
        fs::symlink_metadata(root)?.file_type().is_dir(),
        "Invalid native cache directory"
    );
    fn visit(root: &Path, dir: &Path, hash: &mut sha1::Sha1) -> anyhow::Result<()> {
        let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let path = entry.path();
            let kind = entry.file_type()?;
            ensure!(!kind.is_symlink(), "Native cache contains a symbolic link");
            if kind.is_dir() {
                visit(root, &path, hash)?;
            } else {
                ensure!(kind.is_file(), "Native cache contains an unsupported file");
                let name = path.strip_prefix(root)?.to_string_lossy().into_owned();
                hash.update((name.len() as u64).to_le_bytes());
                hash.update(name.as_bytes());
                let mut file = fs::File::open(&path)?;
                hash.update(file.metadata()?.len().to_le_bytes());
                let mut buffer = [0; 8192];
                loop {
                    let read = file.read(&mut buffer)?;
                    if read == 0 {
                        break;
                    }
                    hash.update(&buffer[..read]);
                }
            }
        }
        Ok(())
    }
    let mut hash = sha1::Sha1::new();
    visit(root, root, &mut hash)?;
    Ok(crate::io::files::sha1_hex(hash.finalize()))
}

fn natives_match(expected: &Path, actual: &Path) -> anyhow::Result<bool> {
    if !fs::symlink_metadata(actual)?.file_type().is_dir() {
        return Ok(false);
    }
    for entry in fs::read_dir(expected)? {
        let entry = entry?;
        let target = actual.join(entry.file_name());
        let expected_type = entry.file_type()?;
        let Ok(actual_type) = fs::symlink_metadata(&target) else {
            return Ok(false);
        };
        if expected_type.is_dir() {
            if !actual_type.file_type().is_dir() || !natives_match(&entry.path(), &target)? {
                return Ok(false);
            }
        } else {
            if !actual_type.file_type().is_file() {
                return Ok(false);
            }
            let mut left = std::io::BufReader::new(fs::File::open(entry.path())?);
            let mut right = std::io::BufReader::new(fs::File::open(target)?);
            let mut a = [0; 8192];
            let mut b = [0; 8192];
            loop {
                let n = left.read(&mut a)?;
                let m = right.read(&mut b)?;
                if n != m || a[..n] != b[..m] {
                    return Ok(false);
                }
                if n == 0 {
                    break;
                }
            }
        }
    }
    Ok(true)
}

pub async fn prepare_launch_libraries(
    details: &McVersion,
    repo: &Path,
    instance: &Path,
    context: &McRuleContext,
    downloader: &Downloader,
    verify: bool,
) -> anyhow::Result<PathBuf> {
    let mut plan = InstallPlan::new();
    plan.libraries(details, repo, context, 2)?;
    add_maven_libraries(&mut plan, details, repo, instance, context, downloader, 2).await?;
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
    let cache = instance
        .join(".pcl-proto")
        .join("natives")
        .join(format!("{}-{}", context.os, context.arch));
    let library_metadata = serde_json::to_vec(&details.libraries)?;
    let context = McRuleContext {
        os: context.os.clone(),
        arch: context.arch.clone(),
        version: context.version.clone(),
        features: context.features.clone(),
    };
    tokio::task::spawn_blocking(move || {
        validate_lwjgl_natives(&plan.natives, &context)?;
        fs::create_dir_all(&cache)?;
        let staging = tempfile::Builder::new()
            .prefix(".staging-")
            .tempdir_in(&cache)?;
        extract_natives(&plan.natives, staging.path())?;
        let mut hasher = sha1::Sha1::new();
        hasher.update(natives_digest(staging.path())?.as_bytes());
        hasher.update(library_metadata);
        let digest = crate::io::files::sha1_hex(hasher.finalize());
        let mut destination = cache.join(&digest);
        if destination.exists() && natives_match(staging.path(), &destination).unwrap_or(false) {
            return Ok(destination);
        }
        if destination.exists() {
            destination = cache.join(format!("{digest}-{}", uuid::Uuid::new_v4()));
        }
        match fs::rename(staging.path(), &destination) {
            Ok(()) => {}
            Err(_error)
                if destination.is_dir()
                    && natives_match(staging.path(), &destination).unwrap_or(false) => {}
            Err(error) => return Err(error.into()),
        }
        Ok::<_, anyhow::Error>(destination)
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

fn merge_loader_profile(
    metadata: &mut serde_json::Value,
    profile: &serde_json::Value,
) -> anyhow::Result<()> {
    profile["libraries"]
        .as_array()
        .context("Invalid loader libraries")?;
    metadata["libraries"]
        .as_array()
        .context("Invalid Minecraft libraries")?;
    ensure!(
        profile["mainClass"]
            .as_str()
            .is_some_and(|name| !name.is_empty()),
        "Invalid loader main class"
    );
    *metadata = crate::minecraft::instance::merge_metadata([profile, &*metadata])?;
    metadata.as_object_mut().unwrap().remove("inheritsFrom");
    Ok(())
}

/// 要安装的模组加载器种类。
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LoaderFamily {
    Fabric,
    Legacyfabric,
    Forge,
    Neoforge,
}

/// 一次加载器安装请求。
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct LoaderInstall {
    pub family: LoaderFamily,
    pub version: String,
    #[cfg(feature = "community")]
    #[serde(default)]
    pub api: Option<FabricApiInstall>,
    #[serde(default)]
    pub install_java: bool,
}

/// 可选一并安装的 Fabric API。
#[cfg(feature = "community")]
#[derive(Clone, Debug, serde::Deserialize, serde::Serialize)]
pub struct FabricApiInstall {
    #[serde(flatten)]
    pub file: DownloadFile,
    pub filename: String,
}

#[cfg(feature = "community")]
fn add_fabric_api(
    plan: &mut InstallPlan,
    instance: &Path,
    api: &FabricApiInstall,
) -> anyhow::Result<()> {
    validate_file_name(&api.filename)?;
    crate::minecraft::export::resource::validate(&api.file)?;
    plan.stages.insert(8, Vec::new());
    plan.add(8, &api.file, instance.join("mods").join(&api.filename))
}

/// 安装完成后的落盘信息；宿主据此更新隔离设置并选择实例。
pub struct InstallOutcome {
    pub directory: std::path::PathBuf,
    pub isolated: bool,
}

#[allow(clippy::too_many_arguments)]
/// 安装版本；`forge_installer` 缺省时使用 core 自带的处理器桥接。
pub async fn install_version(
    client: &MinecraftApiClient,
    repo: &Path,
    version_id: &str,
    name: &str,
    loader: Option<&LoaderInstall>,
    task_id: i32,
    on_event: &EventSink<TaskItemReport>,
    java: &[crate::minecraft::java::JavaEntry],
    java_runtime_dir: &Path,
    forge_installer: Option<&Path>,
) -> anyhow::Result<InstallOutcome> {
    install_version_with_modpack(
        client,
        repo,
        version_id,
        name,
        loader,
        None,
        task_id,
        on_event,
        java,
        java_runtime_dir,
        forge_installer,
    )
    .await
}

// 安装流程的参数本来就是一路透传的上下文，拆成结构体只会多一层解构。
#[allow(clippy::too_many_arguments)]
async fn install_version_with_modpack(
    client: &MinecraftApiClient,
    repo: &Path,
    version_id: &str,
    name: &str,
    loader: Option<&LoaderInstall>,
    pack: Option<&modpack::Modpack>,
    task_id: i32,
    on_event: &EventSink<TaskItemReport>,
    java: &[crate::minecraft::java::JavaEntry],
    java_runtime_dir: &Path,
    forge_installer: Option<&Path>,
) -> anyhow::Result<InstallOutcome> {
    validate_file_name(version_id)?;
    ensure!(
        repo.is_absolute() && repo.is_dir(),
        "Minecraft folder must be an existing absolute directory"
    );
    let repo = repo.canonicalize()?;
    let directory = InstallDirectory::create(&repo, name)?;
    log::info!(
        "Installing Minecraft {version_id} as {name} in {}",
        repo.display()
    );
    on_event(TaskItemReport {
        task_id,
        item_id: 0,
        files_remaining: 1,
        progress: 0.0,
        status: TaskStatus::Running,
        speed: None,
    });
    let mut metadata = client.get_version_metadata(version_id).await?;
    let downloader = client.downloader().await;
    let mut forgelike = None;
    if let Some(loader) = loader {
        on_event(TaskItemReport {
            task_id,
            item_id: 5,
            files_remaining: 1,
            progress: 0.0,
            status: TaskStatus::Running,
            speed: None,
        });
        match loader.family {
            LoaderFamily::Fabric | LoaderFamily::Legacyfabric => {
                let family = match loader.family {
                    LoaderFamily::Fabric => crate::io::net::http::api_client::FabricFamily::Fabric,
                    _ => crate::io::net::http::api_client::FabricFamily::Legacyfabric,
                };
                let profile = client
                    .get_fabric_profile(version_id, &loader.version, family)
                    .await?;
                on_event(TaskItemReport {
                    task_id,
                    item_id: 5,
                    files_remaining: 0,
                    progress: 1.0,
                    status: TaskStatus::Completed,
                    speed: None,
                });
                on_event(TaskItemReport {
                    task_id,
                    item_id: 6,
                    files_remaining: 1,
                    progress: 0.0,
                    status: TaskStatus::Running,
                    speed: None,
                });
                merge_loader_profile(&mut metadata, &profile)?;
            }
            LoaderFamily::Forge | LoaderFamily::Neoforge => {
                #[cfg(feature = "community")]
                ensure!(loader.api.is_none(), "Fabric API requires Fabric");
                forgelike = Some(
                    forgelike::prepare(client, &downloader, version_id, loader, task_id, on_event)
                        .await?,
                );
            }
        }
    }
    let processor_java = if forgelike
        .as_ref()
        .is_some_and(|installer| installer.profile.kind == forgelike::InstallerKind::Processors)
    {
        Some(
            forgelike::ensure_java(
                client,
                &metadata,
                version_id,
                loader.is_some_and(|loader| loader.install_java),
                task_id,
                on_event,
                java,
                java_runtime_dir,
            )
            .await?,
        )
    } else {
        None
    };
    let details: McVersion =
        serde_json::from_value(metadata.clone()).context("Invalid vanilla version metadata")?;
    validate_file_name(&details.asset_index.id)?;
    details.asset_index.download.validate()?;
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
    plan.libraries(&details, &repo, &McRuleContext::current(), 2)?;
    add_maven_libraries(
        &mut plan,
        &details,
        &repo,
        &directory.path,
        &McRuleContext::current(),
        &downloader,
        2,
    )
    .await?;
    #[cfg(feature = "community")]
    if let Some(api) = loader.and_then(|loader| loader.api.as_ref()) {
        add_fabric_api(&mut plan, &directory.path, api)?;
    }
    if loader.is_some() && forgelike.is_none() {
        on_event(TaskItemReport {
            task_id,
            item_id: 6,
            files_remaining: 0,
            progress: 1.0,
            status: TaskStatus::Completed,
            speed: None,
        });
    }
    plan.assets(&index, &repo, "https://resources.download.minecraft.net")?;
    on_event(TaskItemReport {
        task_id,
        item_id: 0,
        files_remaining: 0,
        progress: 1.0,
        status: TaskStatus::Completed,
        speed: None,
    });
    download_plan(&mut plan, &downloader, task_id, on_event).await?;
    if let Some(prepared) = forgelike {
        forgelike::prepare_libraries(
            &prepared,
            &details,
            &metadata,
            &downloader,
            task_id,
            on_event,
        )
        .await?;
        let profile = if prepared.profile.kind == forgelike::InstallerKind::Processors {
            forgelike::run_processors(
                &prepared,
                processor_java.as_ref().context("Missing installer Java")?,
                &metadata,
                version_id,
                &directory.path.join(format!("{name}.jar")),
                task_id,
                on_event,
                forge_installer,
            )
            .await?
        } else {
            on_event(TaskItemReport {
                task_id,
                item_id: 9,
                files_remaining: 0,
                progress: 1.0,
                status: TaskStatus::Completed,
                speed: None,
            });
            prepared.profile.version.clone()
        };
        let source = prepared.directory.path().join("libraries");
        let target = repo.clone();
        crate::io::download::tasks::blocking(task_id, move || {
            forgelike::publish_libraries(&source, &target)
        })
        .await??;
        merge_loader_profile(&mut metadata, &profile)?;
    }
    on_event(TaskItemReport {
        task_id,
        item_id: 7,
        files_remaining: 1,
        progress: 0.0,
        status: TaskStatus::Running,
        speed: None,
    });
    let instance_path = directory.path.clone();
    let index_id = details.asset_index.id;
    crate::io::download::tasks::blocking(task_id, move || {
        extract_natives(&plan.natives, &instance_path.join("natives"))?;
        materialize_assets(&index, &repo, &instance_path, &index_id, true)
    })
    .await??;
    if let Some(pack) = pack {
        pack.install_files(&directory.path, name, &downloader, task_id, on_event)
            .await?;
    }
    // CE preserves the source version separately when the instance has a custom name.
    metadata["id"] = name.into();
    metadata["clientVersion"] = version_id.into();
    let json = serde_json::to_vec_pretty(&metadata)?;
    let filename = format!("{name}.json");
    let isolated = pack.is_some();
    #[cfg(feature = "community")]
    let isolated = isolated || loader.and_then(|loader| loader.api.as_ref()).is_some();
    let path = directory.path.clone();
    crate::io::download::tasks::commit(task_id, move || {
        let mut directory = directory;
        let temporary_json = directory.path.join(".pcl-version.json.part");
        fs::write(&temporary_json, json)?;
        fs::rename(temporary_json, directory.path.join(filename))?;
        directory.committed = true;
        Ok(())
    })
    .await?;
    on_event(TaskItemReport {
        task_id,
        item_id: 7,
        files_remaining: 0,
        progress: 1.0,
        status: TaskStatus::Completed,
        speed: None,
    });
    log::info!("Installed Minecraft {version_id}: {}", path.display());
    Ok(InstallOutcome {
        directory: path,
        isolated,
    })
}

/// 安装一个版本并把结果交给宿主处理（选择实例、记录隔离设置）。
#[allow(clippy::too_many_arguments)]
pub async fn download_minecraft_version(
    repo: &Path,
    client: &MinecraftApiClient,
    on_event: EventSink<TaskItemReport>,
    version_id: String,
    task_id: i32,
    instance_name: String,
    loader: Option<LoaderInstall>,
    java: &[crate::minecraft::java::JavaEntry],
    java_runtime_dir: &Path,
    forge_installer: Option<&Path>,
) -> Result<InstallOutcome, String> {
    crate::io::download::tasks::run(task_id, async {
        install_version(
            client,
            repo,
            &version_id,
            &instance_name,
            loader.as_ref(),
            task_id,
            &on_event,
            java,
            java_runtime_dir,
            forge_installer,
        )
        .await
        .map_err(|error| format!("{error:#}"))
    })
    .await
}

#[cfg(test)]
pub(crate) async fn install_version_for_test(
    client: &MinecraftApiClient,
    repo: &Path,
    version_id: &str,
    name: &str,
    loader: Option<&LoaderInstall>,
    task_id: i32,
    on_event: &EventSink<TaskItemReport>,
) -> anyhow::Result<InstallOutcome> {
    let paths = std::env::temp_dir().join("pcl-test-jre");
    install_version(
        client,
        repo,
        version_id,
        name,
        loader,
        task_id,
        on_event,
        &[],
        &paths,
        None,
    )
    .await
}

#[cfg(test)]
mod tests;
