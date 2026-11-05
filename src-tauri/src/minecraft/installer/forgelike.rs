//! Forge and NeoForge installer archives and their isolated installation data.

use anyhow::{Context, ensure};
use serde_json::Value;
use std::{fs, io::Read, path::Path};

const MAX_JSON_BYTES: u64 = 4 * 1024 * 1024;
const MAX_ENTRY_BYTES: u64 = 512 * 1024 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;

#[derive(Debug)]
pub(super) struct InstallerFile {
    pub url: String,
    pub hash: Option<String>,
}

pub(super) struct PreparedInstaller {
    pub directory: tempfile::TempDir,
    pub archive: std::path::PathBuf,
    pub profile: InstallerProfile,
}

pub(super) async fn prepare(
    client: &super::MinecraftApiClient,
    downloader: &super::Downloader,
    game: &str,
    loader: &super::LoaderInstall,
    task_id: i32,
    on_event: &tauri::ipc::Channel<super::TaskItemReport>,
) -> anyhow::Result<PreparedInstaller> {
    let file = resolve_installer(client, game, loader).await?;
    let directory = tempfile::tempdir()?;
    let archive = directory.path().join("installer.jar");
    download_installer(&file, downloader, &archive, task_id, 5, on_event).await?;
    let archive_path = archive.clone();
    let libraries = directory.path().join("libraries");
    let game = game.to_owned();
    let profile = crate::io::download::tasks::blocking(task_id, move || {
        read_installer(&archive_path, &game, &libraries)
    })
    .await??;
    Ok(PreparedInstaller {
        directory,
        archive,
        profile,
    })
}

async fn download_installer(
    file: &InstallerFile,
    downloader: &super::Downloader,
    archive: &Path,
    task_id: i32,
    item_id: i32,
    on_event: &tauri::ipc::Channel<super::TaskItemReport>,
) -> anyhow::Result<()> {
    let sha1 = file
        .hash
        .as_ref()
        .filter(|hash| hash.len() == 40)
        .cloned()
        .unwrap_or_default();
    downloader
        .download_with_progress(
            super::DownloadConfig {
                file_index: 0,
                out_path: archive.to_owned(),
                task_item_id: item_id,
                info: super::DownloadFile {
                    url: file.url.clone(),
                    sha1,
                    size: 0,
                    path: None,
                },
            },
            |bytes, total, speed| {
                let _ = on_event.send(super::TaskItemReport {
                    task_id,
                    item_id,
                    files_remaining: 1,
                    progress: total
                        .filter(|total| *total > 0)
                        .map_or(0.0, |total| (bytes as f64 / total as f64).min(1.0)),
                    status: super::TaskStatus::Running,
                    speed: Some(speed),
                });
            },
        )
        .await?;
    let archive_path = archive.to_owned();
    let hash = file.hash.clone();
    crate::io::download::tasks::blocking(task_id, move || -> anyhow::Result<()> {
        ensure!(
            fs::metadata(&archive_path)?.len() >= 64 * 1024,
            "Forge installer is too small"
        );
        if let Some(expected) = hash.as_deref().filter(|hash| hash.len() == 32) {
            let mut input = fs::File::open(&archive_path)?;
            let mut digest = md5::Context::new();
            let mut buffer = [0_u8; 64 * 1024];
            loop {
                let read = input.read(&mut buffer)?;
                if read == 0 {
                    break;
                }
                digest.consume(&buffer[..read]);
            }
            ensure!(
                format!("{:x}", digest.compute()).eq_ignore_ascii_case(expected),
                "Forge installer MD5 mismatch"
            );
        }
        Ok(())
    })
    .await??;
    let _ = on_event.send(super::TaskItemReport {
        task_id,
        item_id,
        files_remaining: 0,
        progress: 1.0,
        status: super::TaskStatus::Completed,
        speed: None,
    });
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn forgelike_installer_name(
    client: tauri::State<'_, &super::MinecraftApiClient>,
    game: String,
    loader: super::LoaderInstall,
) -> Result<String, String> {
    let file = resolve_installer(&client, &game, &loader)
        .await
        .map_err(|e| format!("{e:#}"))?;
    file.url
        .rsplit('/')
        .next()
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .ok_or_else(|| "Invalid installer filename".into())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_forgelike_installer(
    client: tauri::State<'_, &super::MinecraftApiClient>,
    game: String,
    loader: super::LoaderInstall,
    destination: std::path::PathBuf,
    task_id: i32,
    on_event: tauri::ipc::Channel<super::TaskItemReport>,
) -> Result<(), String> {
    crate::io::download::tasks::run(task_id, async {
        async {
            let parent = destination
                .parent()
                .filter(|p| p.is_absolute() && p.is_dir())
                .context("Invalid destination")?;
            let name = destination
                .file_name()
                .and_then(|name| name.to_str())
                .context("Invalid destination")?;
            crate::io::files::validate_file_name(name)?;
            ensure!(!destination.try_exists()?, "Destination already exists");
            let file = resolve_installer(&client, &game, &loader).await?;
            let directory = tempfile::tempdir_in(parent)?;
            let archive = directory.path().join("installer");
            download_installer(
                &file,
                &super::Downloader::default(),
                &archive,
                task_id,
                0,
                &on_event,
            )
            .await?;
            crate::io::download::tasks::commit(task_id, move || {
                let _directory = directory;
                fs::hard_link(&archive, &destination)
                    .context("Cannot publish download (the destination may already exist)")
            })
            .await?;
            Ok::<(), anyhow::Error>(())
        }
        .await
        .map_err(|e| format!("{e:#}"))
    })
    .await
}

pub(super) async fn prepare_libraries(
    prepared: &PreparedInstaller,
    vanilla: &super::McVersion,
    metadata: &Value,
    downloader: &super::Downloader,
    task_id: i32,
    on_event: &tauri::ipc::Channel<super::TaskItemReport>,
) -> anyhow::Result<()> {
    let root = prepared.directory.path();
    let mut details = vanilla.clone();
    details.logging = None;
    let mut libraries = Vec::new();
    for value in [
        prepared.profile.install.get("libraries"),
        prepared.profile.version.get("libraries"),
    ]
    .into_iter()
    .flatten()
    {
        for value in value.as_array().context("Invalid installer libraries")? {
            let mut library: crate::minecraft::version::McLibrary =
                serde_json::from_value(value.clone())?;
            if library.downloads.is_none() && library.url.is_none() {
                library.url = Some("https://libraries.minecraft.net/".into());
            }
            if library.downloads.is_none()
                && library
                    .classpath_path(root, root)?
                    .is_some_and(|path| path.is_file())
            {
                continue;
            }
            libraries.push(library);
        }
    }
    details.libraries = libraries;
    let context = super::McRuleContext::current();
    let mut plan = super::InstallPlan::new();
    plan.stages.clear();
    plan.stages.insert(6, Vec::new());
    plan.libraries(&details, root, &context, 6)?;
    super::add_maven_libraries(&mut plan, &details, root, root, &context, downloader, 6).await?;
    if let Some(mapping) = prepared
        .profile
        .install
        .pointer("/data/MOJMAPS/client")
        .and_then(Value::as_str)
    {
        let coordinate = mapping
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .context("Invalid Mojang mappings coordinate")?;
        let artifact = crate::minecraft::maven::MavenArtifact::parse(coordinate)?;
        let destination =
            crate::io::files::safe_join(&root.join("libraries"), &artifact.relative_slash_path())?;
        let download: super::DownloadFile = serde_json::from_value(
            metadata
                .pointer("/downloads/client_mappings")
                .cloned()
                .context("Minecraft mappings are missing")?,
        )?;
        plan.add(6, &download, destination)?;
    }
    super::download_plan(&mut plan, downloader, task_id, on_event).await
}

fn java_requirements(
    metadata: &Value,
    game: &str,
) -> crate::minecraft::java::requirements::JavaRequirements {
    use crate::minecraft::java::{JavaVersion, requirements::JavaRequirements};
    let mut requirements = JavaRequirements::from_metadata(metadata, game);
    requirements.min = requirements.min.max(JavaVersion(8, 0, 60, 0));
    requirements
}

fn downloadable_java(
    requirements: &crate::minecraft::java::requirements::JavaRequirements,
) -> Option<crate::minecraft::java::requirements::JavaRequirements> {
    use crate::minecraft::java::JavaVersion;
    let mut downloadable = requirements.clone();
    if downloadable.min.0 == 8 && downloadable.accepts(JavaVersion::major(17)) {
        downloadable.min = JavaVersion::major(17);
        downloadable.component = None;
    }
    downloadable.download_major()?;
    Some(downloadable)
}

pub(super) async fn select_java(
    requirements: crate::minecraft::java::requirements::JavaRequirements,
    task_id: i32,
) -> anyhow::Result<Option<crate::minecraft::java::JavaEntry>> {
    use crate::minecraft::java::preference::{self, InstanceJavaSettings};
    let known = crate::minecraft::java::refresh(false)
        .await
        .map_err(anyhow::Error::msg)?;
    let global = crate::app::config::ConfigManager::instance()
        .app_state
        .lock()
        .await
        .launcher_setup
        .selected_java
        .clone();
    crate::io::download::tasks::blocking(task_id, move || {
        let selected = preference::select(
            &known,
            global.as_deref(),
            &InstanceJavaSettings::default(),
            &requirements,
        )
        .0;
        let selected = selected
            .filter(|java| requirements.accepts(java.parsed_version()))
            .or_else(|| {
                preference::select(
                    &known,
                    None,
                    &InstanceJavaSettings::default(),
                    &requirements,
                )
                .0
            });
        selected
    })
    .await
    .map_err(Into::into)
}

#[derive(serde::Serialize)]
pub struct JavaInstallOffer {
    requirements: crate::minecraft::java::requirements::JavaRequirements,
    download_major: Option<u32>,
}

fn requires_java(loader: &super::LoaderInstall, game: &str) -> anyhow::Result<bool> {
    match loader.family {
        super::LoaderFamily::Neoforge => Ok(true),
        super::LoaderFamily::Forge => {
            let version = loader
                .version
                .strip_prefix(&format!("{game}-"))
                .unwrap_or(&loader.version);
            Ok(version
                .split('.')
                .next()
                .and_then(|major| major.parse::<u32>().ok())
                .is_none_or(|major| major >= 20))
        }
        _ => anyhow::bail!("Not a Forge loader"),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn prepare_forgelike_java(
    client: tauri::State<'_, &super::MinecraftApiClient>,
    version_id: String,
    loader: super::LoaderInstall,
) -> Result<Option<JavaInstallOffer>, String> {
    if !requires_java(&loader, &version_id).map_err(|e| e.to_string())? {
        return Ok(None);
    }
    let metadata = client
        .get_version_metadata(&version_id)
        .await
        .map_err(|e| e.to_string())?;
    let requirements = java_requirements(&metadata, &version_id);
    if select_java(requirements.clone(), -1)
        .await
        .map_err(|e| e.to_string())?
        .is_some()
    {
        return Ok(None);
    }
    let download_major = downloadable_java(&requirements).and_then(|value| value.download_major());
    Ok(Some(JavaInstallOffer {
        requirements,
        download_major,
    }))
}

pub(super) async fn ensure_java(
    metadata: &Value,
    game: &str,
    install_java: bool,
    task_id: i32,
    on_event: &tauri::ipc::Channel<super::TaskItemReport>,
) -> anyhow::Result<crate::minecraft::java::JavaEntry> {
    let requirements = java_requirements(metadata, game);
    if let Some(java) = select_java(requirements.clone(), task_id).await? {
        if install_java {
            let _ = on_event.send(super::TaskItemReport {
                task_id,
                item_id: 10,
                files_remaining: 0,
                progress: 1.0,
                status: super::TaskStatus::Completed,
                speed: None,
            });
        }
        return Ok(java);
    }
    ensure!(
        install_java,
        "No compatible Java found for the Forge installer"
    );
    let requirements = downloadable_java(&requirements)
        .context("No compatible Java can be downloaded automatically")?;
    let config = crate::app::config::ConfigManager::instance();
    let channel = on_event.clone();
    let runtime = crate::minecraft::java::install::install(
        requirements,
        config.java_runtime_dir(),
        config.api_client.api_bases_async().await,
        move |progress| {
            let _ = channel.send(super::TaskItemReport {
                task_id,
                item_id: 10,
                files_remaining: progress.total.saturating_sub(progress.completed),
                progress: if progress.total == 0 {
                    0.0
                } else {
                    progress.completed as f64 / progress.total as f64
                },
                status: super::TaskStatus::Running,
                speed: None,
            });
        },
    )
    .await?;
    let runtime = crate::io::download::tasks::blocking(task_id, move || {
        config
            .update(|state| {
                state.java_runtimes.push(runtime.clone());
                crate::minecraft::java::sort_list(&mut state.java_runtimes);
                Ok(runtime)
            })
            .map_err(anyhow::Error::msg)
    })
    .await??;
    let _ = on_event.send(super::TaskItemReport {
        task_id,
        item_id: 10,
        files_remaining: 0,
        progress: 1.0,
        status: super::TaskStatus::Completed,
        speed: None,
    });
    Ok(runtime)
}

struct InstallerProcess {
    child: std::process::Child,
    done: bool,
}

impl Drop for InstallerProcess {
    fn drop(&mut self) {
        if self.done {
            return;
        }
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.child.id() as i32), libc::SIGKILL);
        }
        #[cfg(windows)]
        {
            let _ = std::process::Command::new("taskkill")
                .args(["/PID", &self.child.id().to_string(), "/T", "/F"])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn tail(path: &Path) -> anyhow::Result<String> {
    use std::io::{Seek, SeekFrom};
    let mut file = fs::File::open(path)?;
    let start = file.metadata()?.len().saturating_sub(64 * 1024);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

pub(super) async fn run_processors(
    prepared: &PreparedInstaller,
    java: &crate::minecraft::java::JavaEntry,
    metadata: &Value,
    game: &str,
    vanilla_jar: &Path,
    task_id: i32,
    on_event: &tauri::ipc::Channel<super::TaskItemReport>,
) -> anyhow::Result<Value> {
    let root = prepared.directory.path();
    let vanilla_dir = root.join("versions").join(game);
    fs::create_dir_all(&vanilla_dir)?;
    fs::copy(vanilla_jar, vanilla_dir.join(format!("{game}.jar")))?;
    fs::write(
        vanilla_dir.join(format!("{game}.json")),
        serde_json::to_vec(metadata)?,
    )?;
    fs::write(root.join("launcher_profiles.json"), b"{\"profiles\":{}}")?;
    let bridge = crate::minecraft::launch::settings::forge_installer()?;
    let classpath = std::env::join_paths([bridge.as_path(), prepared.archive.as_path()])?;
    let stdout = root.join("installer.stdout.log");
    let stderr = root.join("installer.stderr.log");
    let mut command = std::process::Command::new(java.java_exe());
    if java.major_version() >= 9 {
        command
            .arg("--add-exports")
            .arg("cpw.mods.bootstraplauncher/cpw.mods.bootstraplauncher=ALL-UNNAMED");
    }
    command
        .arg("-cp")
        .arg(classpath)
        .arg("com.bangbang93.ForgeInstaller")
        .arg(root)
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .stdout(fs::File::create(&stdout)?)
        .stderr(fs::File::create(&stderr)?);
    #[cfg(unix)]
    {
        use std::os::unix::process::CommandExt;
        command.process_group(0);
    }
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        command.creation_flags(0x08000000);
    }
    let mut process = InstallerProcess {
        child: command.spawn().context("Cannot start Forge installer")?,
        done: false,
    };
    let _ = on_event.send(super::TaskItemReport {
        task_id,
        item_id: 9,
        files_remaining: 1,
        progress: 0.0,
        status: super::TaskStatus::Running,
        speed: None,
    });
    let status = loop {
        if let Some(status) = process.child.try_wait()? {
            break status;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    };
    process.done = true;
    let output = tail(&stdout)?;
    let errors = tail(&stderr)?;
    ensure!(
        status.success()
            && output
                .lines()
                .rev()
                .take(5)
                .any(|line| line.trim() == "true"),
        "Forge installer failed ({}): {}\n{}",
        status,
        output
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n"),
        errors
            .lines()
            .rev()
            .take(8)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect::<Vec<_>>()
            .join("\n")
    );
    let id = prepared
        .profile
        .install
        .get("version")
        .and_then(Value::as_str)
        .or_else(|| prepared.profile.version.get("id").and_then(Value::as_str))
        .context("Missing installed loader version ID")?;
    crate::io::files::validate_file_name(id)?;
    let path = root.join("versions").join(id).join(format!("{id}.json"));
    let version: Value = serde_json::from_reader(
        fs::File::open(&path)
            .with_context(|| format!("Installer did not create {}", path.display()))?,
    )?;
    ensure!(
        version["inheritsFrom"].as_str() == Some(game),
        "Installed loader targets a different Minecraft version"
    );
    ensure!(
        version["libraries"].is_array()
            && version["mainClass"].as_str().is_some_and(|s| !s.is_empty()),
        "Invalid installed loader profile"
    );
    let _ = on_event.send(super::TaskItemReport {
        task_id,
        item_id: 9,
        files_remaining: 0,
        progress: 1.0,
        status: super::TaskStatus::Completed,
        speed: None,
    });
    Ok(version)
}

fn same_file(left: &Path, right: &Path) -> anyhow::Result<bool> {
    use sha1::Digest;
    let left_meta = fs::symlink_metadata(left)?;
    let right_meta = fs::symlink_metadata(right)?;
    ensure!(
        left_meta.file_type().is_file() && right_meta.file_type().is_file(),
        "Invalid library file"
    );
    if left_meta.len() != right_meta.len() {
        return Ok(false);
    }
    let mut input = fs::File::open(left)?;
    let mut hash = sha1::Sha1::new();
    let mut buffer = [0_u8; 64 * 1024];
    loop {
        let count = input.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hash.update(&buffer[..count]);
    }
    Ok(crate::io::files::check_sha1(
        right,
        &crate::io::files::sha1_hex(hash.finalize()),
    )?)
}

fn create_library_directory(path: &Path) -> anyhow::Result<()> {
    match fs::create_dir(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            ensure!(
                fs::symlink_metadata(path)?.file_type().is_dir(),
                "Invalid library directory: {}",
                path.display()
            );
            Ok(())
        }
        Err(error) => Err(error.into()),
    }
}

pub(super) fn publish_libraries(source: &Path, repo: &Path) -> anyhow::Result<()> {
    if !source.exists() {
        return Ok(());
    }
    ensure!(
        fs::symlink_metadata(source)?.file_type().is_dir(),
        "Invalid staged libraries"
    );
    let destination = repo.join("libraries");
    create_library_directory(&destination)?;
    let mut directories = vec![(source.to_owned(), destination)];
    let mut total = 0_u64;
    while let Some((from, to)) = directories.pop() {
        for entry in fs::read_dir(from)? {
            let entry = entry?;
            let name = entry.file_name();
            crate::io::files::validate_file_name(
                name.to_str().context("Invalid library filename")?,
            )?;
            let target = to.join(&name);
            let kind = entry.file_type()?;
            if kind.is_dir() {
                create_library_directory(&target)?;
                directories.push((entry.path(), target));
            } else {
                ensure!(
                    kind.is_file(),
                    "Invalid staged library: {}",
                    entry.path().display()
                );
                let size = entry.metadata()?.len();
                ensure!(size <= MAX_ENTRY_BYTES, "Library is too large");
                total = total.checked_add(size).context("Libraries are too large")?;
                ensure!(total <= MAX_ARCHIVE_BYTES, "Libraries are too large");
                if target.exists() {
                    ensure!(
                        same_file(&entry.path(), &target)?,
                        "Library conflicts with an existing file: {}",
                        target.display()
                    );
                    continue;
                }
                let mut temporary = tempfile::NamedTempFile::new_in(&to)?;
                std::io::copy(&mut fs::File::open(entry.path())?, &mut temporary)?;
                match temporary.persist_noclobber(&target) {
                    Ok(_) => (),
                    Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                        ensure!(
                            same_file(&entry.path(), &target)?,
                            "Library conflicts with an existing file: {}",
                            target.display()
                        );
                    }
                    Err(error) => return Err(error.error.into()),
                }
            }
        }
    }
    Ok(())
}

fn version_token(value: &str) -> bool {
    !value.is_empty()
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'+'))
        && value != "."
        && value != ".."
}

pub(super) async fn resolve_installer(
    client: &super::MinecraftApiClient,
    game: &str,
    loader: &super::LoaderInstall,
) -> anyhow::Result<InstallerFile> {
    ensure!(
        version_token(game) && version_token(&loader.version),
        "Invalid loader version"
    );
    if matches!(loader.family, super::LoaderFamily::Neoforge) {
        let (package, raw) = if game == "1.20.1" {
            (
                "forge",
                format!(
                    "1.20.1-{}",
                    loader
                        .version
                        .strip_prefix("1.20.1-")
                        .unwrap_or(&loader.version)
                ),
            )
        } else {
            ("neoforge", loader.version.clone())
        };
        ensure!(
            crate::io::net::http::api_client::neoforge_game_version(&raw).as_deref() == Some(game),
            "NeoForge version does not match Minecraft"
        );
        return Ok(InstallerFile {
            url: format!(
                "https://maven.neoforged.net/releases/net/neoforged/{package}/{raw}/{package}-{raw}-installer.jar"
            ),
            hash: None,
        });
    }
    ensure!(
        matches!(loader.family, super::LoaderFamily::Forge),
        "Not a Forge loader"
    );
    let version = loader
        .version
        .strip_prefix(&format!("{game}-"))
        .unwrap_or(&loader.version);
    let entry = client
        .forge_versions(game, Some(version))
        .await?
        .into_iter()
        .find(|entry| entry.version == version || entry.file_version == version)
        .context("Forge version not found")?;
    Ok(InstallerFile {
        url: entry.installer_url(game),
        hash: entry.hash,
    })
}

#[derive(Debug, PartialEq, Eq)]
pub(super) enum InstallerKind {
    Legacy,
    Processors,
}

pub(super) struct InstallerProfile {
    pub kind: InstallerKind,
    pub install: Value,
    pub version: Value,
}

fn read_json(archive: &mut zip::ZipArchive<fs::File>, name: &str) -> anyhow::Result<Value> {
    let name = name.trim_start_matches('/');
    crate::io::files::safe_join(Path::new("archive"), name)?;
    let entry = archive
        .by_name(name)
        .with_context(|| format!("Missing installer entry {name}"))?;
    ensure!(
        entry.size() <= MAX_JSON_BYTES,
        "Installer JSON is too large: {name}"
    );
    let mut bytes = Vec::new();
    entry.take(MAX_JSON_BYTES + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MAX_JSON_BYTES,
        "Installer JSON is too large: {name}"
    );
    let value: Value = serde_json::from_slice(&bytes)
        .with_context(|| format!("Invalid installer JSON: {name}"))?;
    ensure!(
        value.is_object(),
        "Installer JSON must be an object: {name}"
    );
    Ok(value)
}

fn copy_entry(
    archive: &mut zip::ZipArchive<fs::File>,
    name: &str,
    destination: &Path,
    total: &mut u64,
) -> anyhow::Result<()> {
    let mut entry = archive.by_name(name)?;
    ensure!(entry.is_file(), "Invalid installer file: {name}");
    ensure!(
        entry
            .unix_mode()
            .is_none_or(|mode| mode & 0o170000 != 0o120000),
        "Installer entry is a symlink: {name}"
    );
    ensure!(
        entry.size() <= MAX_ENTRY_BYTES,
        "Installer entry is too large: {name}"
    );
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut output = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)?;
    let mut limited = (&mut entry).take(MAX_ENTRY_BYTES + 1);
    let copied = std::io::copy(&mut limited, &mut output)?;
    ensure!(
        copied <= MAX_ENTRY_BYTES,
        "Installer entry is too large: {name}"
    );
    *total = total
        .checked_add(copied)
        .context("Installer is too large")?;
    ensure!(*total <= MAX_ARCHIVE_BYTES, "Installer is too large");
    Ok(())
}

/// The destination must be an empty, installation-owned temporary directory.
pub(super) fn read_installer(
    installer: &Path,
    game: &str,
    libraries: &Path,
) -> anyhow::Result<InstallerProfile> {
    let mut archive = zip::ZipArchive::new(fs::File::open(installer)?)
        .context("Invalid Forge installer archive")?;
    ensure!(archive.len() <= 100_000, "Too many installer entries");
    let install = read_json(&mut archive, "install_profile.json")?;
    let old = install.get("install").filter(|v| !v.is_null());
    let target = old
        .and_then(|v| v.get("minecraft"))
        .or_else(|| install.get("minecraft"));
    if let Some(target) = target {
        ensure!(
            target.as_str() == Some(game),
            "Installer targets a different Minecraft version"
        );
    }
    let version = if old.is_some() {
        install
            .get("versionInfo")
            .cloned()
            .context("Missing legacy Forge version metadata")?
    } else {
        let name = install
            .get("json")
            .and_then(Value::as_str)
            .unwrap_or("version.json")
            .to_owned();
        read_json(&mut archive, &name)?
    };
    ensure!(version.is_object(), "Invalid loader version metadata");
    if let Some(parent) = version.get("inheritsFrom").filter(|v| !v.is_null()) {
        ensure!(
            parent.as_str() == Some(game),
            "Loader profile targets a different Minecraft version"
        );
    }
    ensure!(
        version.get("libraries").and_then(Value::as_array).is_some(),
        "Missing loader libraries"
    );
    ensure!(
        version
            .get("mainClass")
            .and_then(Value::as_str)
            .is_some_and(|v| !v.is_empty()),
        "Missing loader main class"
    );

    let mut entries = Vec::new();
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        if !entry.is_file() {
            continue;
        }
        let name = entry.name()?;
        if let Some(relative) = name.strip_prefix("maven/") {
            let path = crate::io::files::safe_join(libraries, relative)?;
            entries.push((name.to_string(), path));
        }
    }
    let mut total = 0;
    for (name, destination) in entries {
        copy_entry(&mut archive, &name, &destination, &mut total)?;
    }
    if let Some(old) = old {
        let coordinate = old
            .get("path")
            .and_then(Value::as_str)
            .context("Missing Forge Maven coordinate")?;
        let source = old
            .get("filePath")
            .and_then(Value::as_str)
            .context("Missing Forge archive path")?;
        crate::io::files::safe_join(Path::new("archive"), source)?;
        let artifact = crate::minecraft::maven::MavenArtifact::parse(coordinate)?;
        let destination = crate::io::files::safe_join(libraries, &artifact.relative_slash_path())?;
        copy_entry(&mut archive, source, &destination, &mut total)?;
    }
    Ok(InstallerProfile {
        kind: if old.is_some() || install.get("processors").is_none() {
            InstallerKind::Legacy
        } else {
            InstallerKind::Processors
        },
        install,
        version,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn installer_java_offer_uses_a_compatible_download_and_skips_old_forge() {
        use crate::minecraft::java::JavaVersion;
        let legacy = super::super::LoaderInstall {
            family: super::super::LoaderFamily::Forge,
            version: "14.23.5.2860".into(),
            api: None,
            install_java: false,
        };
        assert!(!requires_java(&legacy, "1.12.2").unwrap());
        let modern = super::super::LoaderInstall {
            version: "1.20.1-47.0.1".into(),
            ..legacy
        };
        assert!(requires_java(&modern, "1.20.1").unwrap());
        let old = java_requirements(&serde_json::json!({}), "1.16.5");
        assert_eq!(downloadable_java(&old).unwrap().download_major(), Some(17));
        let current = java_requirements(&serde_json::json!({}), "1.21.1");
        assert_eq!(
            downloadable_java(&current).unwrap().download_major(),
            Some(21)
        );
        let mut restricted = old;
        restricted.max = JavaVersion::upper(8);
        assert!(downloadable_java(&restricted).is_none());
    }

    fn archive(root: &Path, entries: &[(&str, &str)]) -> std::path::PathBuf {
        let path = root.join("installer.jar");
        let mut zip = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        for (name, content) in entries {
            zip.start_file(*name, zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(content.as_bytes()).unwrap();
        }
        zip.finish().unwrap();
        path
    }

    #[test]
    fn publishes_staged_libraries_without_overwriting_existing_data() {
        let root = tempfile::tempdir().unwrap();
        let staged = root.path().join("stage");
        let repo = root.path().join("repo");
        fs::create_dir_all(staged.join("org/example")).unwrap();
        fs::create_dir(&repo).unwrap();
        fs::write(staged.join("org/example/lib.jar"), b"one").unwrap();
        publish_libraries(&staged, &repo).unwrap();
        publish_libraries(&staged, &repo).unwrap();
        fs::write(staged.join("org/example/lib.jar"), b"two").unwrap();
        assert!(publish_libraries(&staged, &repo).is_err());
        assert_eq!(
            fs::read(repo.join("libraries/org/example/lib.jar")).unwrap(),
            b"one"
        );
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.path(), staged.join("escape")).unwrap();
            assert!(publish_libraries(&staged, &repo).is_err());
        }
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn cancellation_stops_installer_process_tree() {
        use std::os::unix::process::CommandExt;
        let root = tempfile::tempdir().unwrap();
        let ready = root.path().join("ready");
        let escaped = root.path().join("escaped");
        let id = -1201;
        crate::io::download::tasks::prepare_download_task(id).unwrap();
        let (started_tx, started_rx) = tokio::sync::oneshot::channel();
        let job = tokio::spawn(crate::io::download::tasks::run(id, async move {
            let child = std::process::Command::new("sh")
                .arg("-c")
                .arg("(sleep 1; touch \"$2\") & touch \"$1\"; wait")
                .arg("sh")
                .arg(&ready)
                .arg(&escaped)
                .process_group(0)
                .spawn()
                .unwrap();
            let process = InstallerProcess { child, done: false };
            while !ready.exists() {
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
            started_tx.send(()).unwrap();
            std::future::pending::<()>().await;
            drop(process);
            Ok::<(), String>(())
        }));
        started_rx.await.unwrap();
        crate::io::download::tasks::cancel_download_task(id).unwrap();
        assert_eq!(job.await.unwrap(), Err("Download cancelled".into()));
        tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
        assert!(!root.path().join("escaped").exists());
    }

    #[test]
    #[ignore = "Downloads a real installer, Minecraft client and processor dependencies"]
    fn real_installer_processors_smoke() {
        let _ = crate::app::config::ConfigManager::instance();
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(real_installer_smoke());
    }

    async fn real_installer_smoke() {
        use super::super::{
            Downloader, LoaderFamily, LoaderInstall, McVersion, MinecraftApiClient,
        };
        use crate::io::net::http::api_client::ApiProvider;
        let case = std::env::var("PCL_FORGE_SMOKE_CASE").unwrap_or_else(|_| "forge".into());
        let (game, version, family) = match case.as_str() {
            "forge" => ("1.20.1", "47.0.1", LoaderFamily::Forge),
            "neoforge" => ("1.20.4", "20.4.30-beta", LoaderFamily::Neoforge),
            "middle" => ("1.12.2", "14.23.5.2860", LoaderFamily::Forge),
            "legacy" => ("1.7.10", "10.13.0.1151", LoaderFamily::Forge),
            _ => panic!("Unknown smoke case"),
        };
        let client = MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::BMCLApi);
        let downloader = Downloader::with_bases(client.api_bases_async().await);
        let metadata = client.get_version_metadata(game).await.unwrap();
        let vanilla: McVersion = serde_json::from_value(metadata.clone()).unwrap();
        let loader = LoaderInstall {
            family,
            version: version.into(),
            api: None,
            install_java: false,
        };
        let events = tauri::ipc::Channel::new(|_| Ok(()));
        let prepared = prepare(&client, &downloader, game, &loader, 0, &events)
            .await
            .unwrap();
        let temp = tempfile::tempdir().unwrap();
        let jar = temp.path().join("client.jar");
        downloader
            .download(&vanilla.downloads.client, &jar)
            .await
            .unwrap();
        prepare_libraries(&prepared, &vanilla, &metadata, &downloader, 0, &events)
            .await
            .unwrap();
        let profile = if prepared.profile.kind == InstallerKind::Processors {
            let java = select_java(java_requirements(&metadata, game), 0)
                .await
                .unwrap()
                .unwrap();
            run_processors(&prepared, &java, &metadata, game, &jar, 0, &events)
                .await
                .unwrap()
        } else {
            prepared.profile.version.clone()
        };
        publish_libraries(&prepared.directory.path().join("libraries"), temp.path()).unwrap();
        for value in profile["libraries"].as_array().unwrap() {
            let library: crate::minecraft::version::McLibrary =
                serde_json::from_value(value.clone()).unwrap();
            if library
                .rule_allow(&super::super::McRuleContext::current())
                .unwrap()
                && let Some(path) = library.classpath_path(temp.path(), temp.path()).unwrap()
            {
                assert!(
                    path.is_file(),
                    "Missing {} at {}",
                    library.name,
                    path.display()
                );
            }
        }
        let mut combined = metadata;
        super::super::merge_loader_profile(&mut combined, &profile).unwrap();
        assert!(
            combined["mainClass"]
                .as_str()
                .is_some_and(|v| !v.is_empty())
        );
        assert!(combined.get("inheritsFrom").is_none());
        combined["id"] = "Loader Test".into();
        combined["clientVersion"] = game.into();
        let folder = temp.path().join("versions/Loader Test");
        fs::create_dir_all(&folder).unwrap();
        fs::write(
            folder.join("Loader Test.json"),
            serde_json::to_vec(&combined).unwrap(),
        )
        .unwrap();
        fs::copy(&jar, folder.join("Loader Test.jar")).unwrap();
        let repository =
            crate::minecraft::instance::folder::McFolder::new("test", temp.path().to_owned());
        let instance =
            crate::minecraft::instance::McInstance::from_version_folder(&folder, &repository)
                .unwrap();
        assert!(if case == "neoforge" {
            instance.info().has_neoforge
        } else {
            instance.info().has_forge
        });
        if std::env::var_os("PCL_FORGE_SMOKE_FULL").is_some() {
            let repository = tempfile::tempdir().unwrap();
            super::super::install_version(
                &client,
                repository.path(),
                game,
                "Full Loader Test",
                Some(&loader),
                0,
                &events,
            )
            .await
            .unwrap();
            let folder = repository.path().join("versions/Full Loader Test");
            let repo = crate::minecraft::instance::folder::McFolder::new(
                "test",
                repository.path().to_owned(),
            );
            let installed =
                crate::minecraft::instance::McInstance::from_version_folder(&folder, &repo)
                    .unwrap();
            assert!(installed.jar_path.is_file());
            assert!(if case == "neoforge" {
                installed.info().has_neoforge
            } else {
                installed.info().has_forge
            });
            let resolved = installed.resolved_metadata().unwrap();
            let details: McVersion =
                serde_json::from_value(resolved.clone()).unwrap_or_else(|error| {
                    panic!(
                        "Invalid installed metadata ({error}): downloads={}",
                        resolved["downloads"]
                    )
                });
            let context = super::super::McRuleContext::current();
            super::super::InstallPlan::new()
                .libraries(&details, repository.path(), &context, 2)
                .unwrap();
            let mut seen = std::collections::HashSet::new();
            for library in &details.libraries {
                if library.rule_allow(&context).unwrap()
                    && let Some(path) = library.classpath_path(repository.path(), &folder).unwrap()
                    && seen.insert(library.classpath_key().unwrap())
                {
                    assert!(
                        path.is_file(),
                        "Missing {} at {}",
                        library.name,
                        path.display()
                    );
                }
            }
        }
    }

    #[test]
    fn reads_legacy_and_processor_profiles_without_writing_outside_staging() {
        let root = tempfile::tempdir().unwrap();
        let legacy = archive(
            root.path(),
            &[
                (
                    "install_profile.json",
                    r#"{
            "install":{"minecraft":"1.12.2","path":"net.minecraftforge:forge:1.12.2-14.23","filePath":"forge.jar"},
            "versionInfo":{"inheritsFrom":"1.12.2","mainClass":"Main","libraries":[]}
        }"#,
                ),
                ("forge.jar", "legacy"),
            ],
        );
        let libraries = root.path().join("legacy-libraries");
        let profile = read_installer(&legacy, "1.12.2", &libraries).unwrap();
        assert_eq!(profile.kind, InstallerKind::Legacy);
        assert_eq!(
            fs::read(
                libraries.join("net/minecraftforge/forge/1.12.2-14.23/forge-1.12.2-14.23.jar")
            )
            .unwrap(),
            b"legacy"
        );

        let modern = archive(
            root.path(),
            &[
                (
                    "install_profile.json",
                    r#"{
            "minecraft":"1.20.1","json":"/version.json","processors":[]
        }"#,
                ),
                (
                    "version.json",
                    r#"{"inheritsFrom":"1.20.1","mainClass":"Main","libraries":[]}"#,
                ),
                ("maven/net/example/tool/1/tool-1.jar", "tool"),
            ],
        );
        let libraries = root.path().join("modern-libraries");
        let profile = read_installer(&modern, "1.20.1", &libraries).unwrap();
        assert_eq!(profile.kind, InstallerKind::Processors);
        assert_eq!(profile.install["minecraft"], "1.20.1");
        assert_eq!(profile.version["mainClass"], "Main");
        assert_eq!(
            fs::read(libraries.join("net/example/tool/1/tool-1.jar")).unwrap(),
            b"tool"
        );
        assert!(read_installer(&modern, "1.19.4", &root.path().join("wrong-game")).is_err());

        let bad = archive(
            root.path(),
            &[
                ("install_profile.json", r#"{"minecraft":"1.20.1"}"#),
                ("version.json", r#"{"mainClass":"Main","libraries":[]}"#),
                ("maven/../escape", "bad"),
            ],
        );
        assert!(read_installer(&bad, "1.20.1", &root.path().join("bad-libraries")).is_err());
        assert!(!root.path().join("escape").exists());
    }
}
