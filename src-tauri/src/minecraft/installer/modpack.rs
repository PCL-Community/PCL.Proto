use super::*;
use std::{
    collections::{HashMap, HashSet},
    sync::{LazyLock, Mutex as StdMutex},
};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Index {
    format_version: u32,
    game: String,
    name: String,
    dependencies: HashMap<String, String>,
    files: Vec<PackFile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PackFile {
    path: String,
    hashes: HashMap<String, String>,
    downloads: Vec<String>,
    file_size: u64,
    #[serde(default)]
    env: HashMap<String, String>,
}

#[derive(serde::Serialize)]
pub struct PackInfo {
    name: String,
    minecraft: String,
    fabric: bool,
    optional_files: Vec<String>,
}

pub struct Modpack {
    archive: Arc<StdMutex<zip::ZipArchive<fs::File>>>,
    prefix: String,
    index: Index,
    optional: HashSet<String>,
}

struct PreparedPack {
    pack: Modpack,
    _directory: tempfile::TempDir,
}

static PREPARED: LazyLock<StdMutex<HashMap<String, PreparedPack>>> =
    LazyLock::new(|| StdMutex::new(HashMap::new()));

#[derive(serde::Serialize)]
pub struct OnlinePackInfo {
    token: String,
    info: PackInfo,
}

impl Modpack {
    fn open(path: &Path) -> anyhow::Result<Self> {
        ensure!(path.is_absolute() && path.is_file(), "Invalid modpack file");
        let mut archive = zip::ZipArchive::new(fs::File::open(path)?)?;
        let mut indexes = Vec::new();
        for i in 0..archive.len() {
            let entry = archive.by_index(i)?;
            let name = entry.name()?;
            if name == "modrinth.index.json"
                || name
                    .strip_suffix("/modrinth.index.json")
                    .is_some_and(|p| !p.is_empty() && !p.contains('/'))
            {
                indexes.push(name.into_owned());
            }
        }
        ensure!(
            indexes.len() == 1,
            "Unsupported or ambiguous modpack format"
        );
        let prefix = indexes[0]
            .strip_suffix("modrinth.index.json")
            .unwrap()
            .to_owned();
        let index: Index = {
            let entry = archive.by_name(&indexes[0])?;
            ensure!(
                entry.size() <= 4 * 1024 * 1024,
                "Modpack index is too large"
            );
            serde_json::from_reader(entry).context("Invalid Modrinth index")?
        };
        ensure!(
            index.format_version == 1 && index.game == "minecraft",
            "Unsupported Modrinth format or game"
        );
        ensure!(index.files.len() <= 100_000, "Too many modpack files");
        let minecraft = index
            .dependencies
            .get("minecraft")
            .context("Missing Minecraft version")?;
        validate_file_name(minecraft)?;
        let mut loader_count = 0;
        for (key, value) in &index.dependencies {
            if key == "minecraft" {
                continue;
            }
            ensure!(key == "fabric-loader", "Unsupported modpack loader: {key}");
            validate_file_name(value)?;
            loader_count += 1;
        }
        ensure!(loader_count <= 1, "Conflicting modpack loaders");
        let mut paths = HashSet::new();
        for file in &index.files {
            safe_join(Path::new("instance"), &file.path)?;
            ensure!(
                paths.insert(file.path.to_ascii_lowercase()),
                "Duplicate modpack file path"
            );
            ensure!(
                matches!(
                    file.env.get("client").map(String::as_str),
                    None | Some("required" | "optional" | "unsupported")
                ),
                "Invalid client environment"
            );
            ensure!(
                file.file_size <= 4 * 1024 * 1024 * 1024,
                "Modpack file is too large"
            );
            let sha1 = file.hashes.get("sha1").context("Missing modpack SHA-1")?;
            ensure!(!file.downloads.is_empty(), "Missing modpack download URLs");
            for url in &file.downloads {
                DownloadFile {
                    url: url.clone(),
                    sha1: sha1.clone(),
                    size: file.file_size,
                    path: None,
                }
                .validate()?;
            }
        }
        Ok(Self {
            archive: Arc::new(StdMutex::new(archive)),
            prefix,
            index,
            optional: HashSet::new(),
        })
    }

    fn info(&self) -> PackInfo {
        PackInfo {
            name: self.index.name.clone(),
            minecraft: self.index.dependencies["minecraft"].clone(),
            fabric: self.index.dependencies.contains_key("fabric-loader"),
            optional_files: self
                .index
                .files
                .iter()
                .filter(|file| file.env.get("client").is_some_and(|v| v == "optional"))
                .map(|file| file.path.clone())
                .collect(),
        }
    }

    pub(super) async fn install_files(
        &self,
        directory: &Path,
        name: &str,
        downloader: &Downloader,
        task_id: i32,
        on_event: &Channel<TaskItemReport>,
    ) -> anyhow::Result<()> {
        let files: Vec<_> = self
            .index
            .files
            .iter()
            .filter(|file| match file.env.get("client").map(String::as_str) {
                Some("unsupported") => false,
                Some("optional") => self.optional.contains(&file.path),
                _ => true,
            })
            .collect();
        let total = files.len();
        for (i, file) in files.into_iter().enumerate() {
            let path = pack_path(directory, name, &file.path)?;
            fs::create_dir_all(path.parent().context("Missing modpack parent")?)?;
            let mut last_error = None;
            for url in &file.downloads {
                let info = DownloadFile {
                    url: url.clone(),
                    sha1: file.hashes["sha1"].clone(),
                    size: file.file_size,
                    path: None,
                };
                match downloader.download(&info, &path).await {
                    Ok(()) => {
                        last_error = None;
                        break;
                    }
                    Err(error) => last_error = Some(error),
                }
            }
            if let Some(error) = last_error {
                return Err(error).with_context(|| format!("Cannot download {}", file.path));
            }
            let _ = on_event.send(TaskItemReport {
                task_id,
                item_id: 4,
                files_remaining: total - i - 1,
                progress: (i + 1) as f64 / total as f64,
                status: TaskStatus::Running,
                speed: None,
            });
        }
        let archive = self.archive.clone();
        let prefix = self.prefix.clone();
        let directory = directory.to_owned();
        let name = name.to_owned();
        crate::io::download::tasks::blocking(task_id, move || {
            extract_overrides(&archive, &prefix, &directory, &name)
        })
        .await??;
        let _ = on_event.send(TaskItemReport {
            task_id,
            item_id: 4,
            files_remaining: 0,
            progress: 1.0,
            status: TaskStatus::Completed,
            speed: None,
        });
        Ok(())
    }
}

fn pack_path(directory: &Path, name: &str, relative: &str) -> anyhow::Result<PathBuf> {
    let first = relative.split('/').next().unwrap_or("");
    ensure!(
        !first.eq_ignore_ascii_case("natives")
            && !first.eq_ignore_ascii_case(&format!("{name}.jar"))
            && !first.eq_ignore_ascii_case(&format!("{name}.json")),
        "Modpack path conflicts with game files"
    );
    safe_join(directory, relative)
}

fn extract_overrides(
    archive: &StdMutex<zip::ZipArchive<fs::File>>,
    prefix: &str,
    directory: &Path,
    name: &str,
) -> anyhow::Result<()> {
    let mut archive = archive
        .lock()
        .map_err(|_| anyhow::anyhow!("Cannot access modpack"))?;
    let mut total = 0_u64;
    for folder in ["overrides/", "client-overrides/"] {
        let prefix = format!("{prefix}{folder}");
        let mut seen = HashSet::new();
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i)?;
            let entry_name = entry.name()?.to_owned();
            let Some(relative) = entry_name.strip_prefix(&prefix) else {
                continue;
            };
            if entry.is_dir() {
                continue;
            }
            ensure!(
                entry
                    .unix_mode()
                    .is_none_or(|mode| mode & 0o170000 != 0o120000),
                "Modpack contains a symbolic link"
            );
            ensure!(
                seen.insert(relative.to_ascii_lowercase()),
                "Duplicate override path"
            );
            // 额度按实际写入字节累计：archive 头声明的 size 可以被伪造。
            const MAX_ENTRY: u64 = 512 * 1024 * 1024;
            const MAX_TOTAL: u64 = 4 * 1024 * 1024 * 1024;
            ensure!(total < MAX_TOTAL, "Modpack overrides are too large");
            let target = pack_path(directory, name, relative)?;
            fs::create_dir_all(target.parent().context("Missing override parent")?)?;
            let mut output = fs::File::create(target)?;
            total += crate::io::files::copy_entry_bounded(
                &mut entry,
                &mut output,
                MAX_ENTRY.min(MAX_TOTAL - total),
            )?;
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn inspect_modpack(path: PathBuf) -> Result<PackInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        Modpack::open(&path)
            .map(|pack| pack.info())
            .map_err(|error| format!("{error:#}"))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(rename_all = "snake_case")]
pub async fn prepare_online_modpack(
    info: DownloadFile,
    task_id: i32,
    on_event: Channel<TaskItemReport>,
) -> Result<OnlinePackInfo, String> {
    let prepared = crate::io::download::tasks::run(task_id, async {
        prepare_online_archive(info, task_id, &on_event)
            .await
            .map_err(|e| format!("{e:#}"))
    })
    .await?;
    let info = prepared.pack.info();
    let token = uuid::Uuid::new_v4().to_string();
    PREPARED
        .lock()
        .map_err(|e| e.to_string())?
        .insert(token.clone(), prepared);
    Ok(OnlinePackInfo { token, info })
}

async fn prepare_online_archive(
    info: DownloadFile,
    task_id: i32,
    on_event: &Channel<TaskItemReport>,
) -> anyhow::Result<PreparedPack> {
    crate::minecraft::export::resource::validate(&info)?;
    let directory = tempfile::tempdir()?;
    let path = directory.path().join("modpack.zip");
    crate::minecraft::export::resource::download_to(info, path.clone(), task_id, on_event).await?;
    crate::io::download::tasks::blocking(task_id, move || {
        let pack = Modpack::open(&path)?;
        Ok(PreparedPack {
            pack,
            _directory: directory,
        })
    })
    .await?
}

#[tauri::command]
pub fn release_online_modpack(token: String) {
    if let Ok(mut prepared) = PREPARED.lock() {
        prepared.remove(&token);
    }
}

async fn install_pack(
    mut pack: Modpack,
    repo: &Path,
    client: &MinecraftApiClient,
    instance_name: &str,
    optional_files: Vec<String>,
    task_id: i32,
    on_event: &Channel<TaskItemReport>,
) -> anyhow::Result<()> {
    let available: HashSet<_> = pack.info().optional_files.into_iter().collect();
    ensure_optional(&available, &optional_files)?;
    pack.optional = optional_files.into_iter().collect();
    let fabric = pack
        .index
        .dependencies
        .get("fabric-loader")
        .map(|version| FabricInstall {
            family: crate::io::net::http::api_client::FabricFamily::Fabric,
            version: version.clone(),
            api: None,
        });
    install_version_with_modpack(
        client,
        repo,
        &pack.index.dependencies["minecraft"],
        instance_name,
        fabric.as_ref(),
        Some(&pack),
        task_id,
        on_event,
    )
    .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn install_modpack(
    state: tauri::State<'_, Arc<Mutex<crate::app::config::AppState>>>,
    client: tauri::State<'_, &MinecraftApiClient>,
    path: PathBuf,
    instance_name: String,
    optional_files: Vec<String>,
    task_id: i32,
    on_event: Channel<TaskItemReport>,
) -> Result<(), String> {
    crate::io::download::tasks::run(task_id, async {
        let pack = crate::io::download::tasks::blocking(task_id, move || Modpack::open(&path))
            .await
            .map_err(|error| error.to_string())?
            .map_err(|error| format!("{error:#}"))?;
        let repo = state.lock().await.active_repo_path.clone();
        install_pack(
            pack,
            &repo,
            &client,
            &instance_name,
            optional_files,
            task_id,
            &on_event,
        )
        .await
        .map_err(|error| format!("{error:#}"))
    })
    .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn install_online_modpack(
    state: tauri::State<'_, Arc<Mutex<crate::app::config::AppState>>>,
    client: tauri::State<'_, &MinecraftApiClient>,
    token: String,
    instance_name: String,
    optional_files: Vec<String>,
    task_id: i32,
    on_event: Channel<TaskItemReport>,
) -> Result<(), String> {
    let prepared = PREPARED
        .lock()
        .map_err(|e| e.to_string())?
        .remove(&token)
        .ok_or("Modpack preparation has expired")?;
    crate::io::download::tasks::run(task_id, async {
        let PreparedPack { pack, _directory } = prepared;
        let repo = state.lock().await.active_repo_path.clone();
        install_pack(
            pack,
            &repo,
            &client,
            &instance_name,
            optional_files,
            task_id,
            &on_event,
        )
        .await
        .map_err(|e| format!("{e:#}"))
    })
    .await
}

fn ensure_optional(available: &HashSet<String>, selected: &[String]) -> anyhow::Result<()> {
    ensure!(
        selected.iter().all(|path| available.contains(path)),
        "Unknown optional modpack file"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn archive(
        index: serde_json::Value,
        prefix: &str,
        overrides: &[(&str, &[u8])],
    ) -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("test.mrpack");
        let mut zip = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        zip.start_file(
            format!("{prefix}modrinth.index.json"),
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.write_all(index.to_string().as_bytes()).unwrap();
        for (name, data) in overrides {
            zip.start_file(
                format!("{prefix}{name}"),
                zip::write::SimpleFileOptions::default(),
            )
            .unwrap();
            zip.write_all(data).unwrap();
        }
        zip.finish().unwrap();
        (temp, path)
    }

    fn index() -> serde_json::Value {
        serde_json::json!({"formatVersion": 1, "game": "minecraft", "name": "Example", "dependencies": {"minecraft": "1.20.1", "fabric-loader": "0.16.0"}, "files": [{"path": "mods/example.jar", "hashes": {"sha1": "0000000000000000000000000000000000000000"}, "downloads": ["https://example.org/example.jar"], "fileSize": 1, "env": {"client": "optional"}}]})
    }

    #[test]
    fn inspect_validates_paths_loaders_and_optional_files() {
        let (_temp, path) = archive(index(), "nested/", &[]);
        let pack = Modpack::open(&path).unwrap();
        assert!(pack.info().fabric);
        assert_eq!(pack.info().optional_files, ["mods/example.jar"]);
        assert!(
            ensure_optional(
                &pack.info().optional_files.into_iter().collect(),
                &["unknown".into()]
            )
            .is_err()
        );
        let mut bad = index();
        bad["files"][0]["path"] = "../escape".into();
        let (_temp, path) = archive(bad, "", &[]);
        assert!(Modpack::open(&path).is_err());
        let mut bad = index();
        bad["dependencies"]["forge"] = "1".into();
        let (_temp, path) = archive(bad, "", &[]);
        assert!(Modpack::open(&path).is_err());
    }

    #[test]
    fn client_overrides_replace_common_overrides_and_cannot_escape() {
        let (temp, path) = archive(
            index(),
            "",
            &[
                ("overrides/config/a", b"common"),
                ("client-overrides/config/a", b"client"),
            ],
        );
        let pack = Modpack::open(&path).unwrap();
        let target = temp.path().join("instance");
        fs::create_dir(&target).unwrap();
        extract_overrides(&pack.archive, &pack.prefix, &target, "instance").unwrap();
        assert_eq!(fs::read(target.join("config/a")).unwrap(), b"client");
        let (_temp, path) = archive(index(), "", &[("overrides/../escape", b"bad")]);
        let pack = Modpack::open(&path).unwrap();
        assert!(extract_overrides(&pack.archive, &pack.prefix, &target, "instance").is_err());
    }

    #[test]
    fn releasing_prepared_pack_removes_its_temporary_directory() {
        let (directory, path) = archive(index(), "", &[]);
        let pack = Modpack::open(&path).unwrap();
        let token = uuid::Uuid::new_v4().to_string();
        PREPARED.lock().unwrap().insert(
            token.clone(),
            PreparedPack {
                pack,
                _directory: directory,
            },
        );
        assert!(path.exists());
        release_online_modpack(token.clone());
        assert!(!path.exists());
        release_online_modpack(token);
    }

    #[test]
    fn online_prepare_ipc_accepts_task_arguments_and_validates_source() {
        let app = tauri::test::mock_builder()
            .invoke_handler(tauri::generate_handler![prepare_online_modpack])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let result = tauri::test::get_ipc_response(
            &window,
            tauri::webview::InvokeRequest {
                cmd: "prepare_online_modpack".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({
                    "info": { "url": "https://example.org/pack.mrpack", "size": 1, "sha1": "", "path": null },
                    "task_id": -2001, "version_id": "Pack", "on_event": "__CHANNEL__:1"
                })),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );
        assert_eq!(result.unwrap_err(), "Unsupported resource host");
    }

    #[tokio::test]
    #[ignore = "Downloads a public Modrinth pack and a complete Minecraft installation"]
    async fn online_fabric_pack_installs_in_isolated_configuration() {
        let root = std::env::var_os("PCL_TEST_CONFIG_ROOT").expect("Set an isolated config root");
        let root = fs::canonicalize(root).unwrap();
        let info = DownloadFile {
            url: "https://cdn.modrinth.com/data/BHtwz1lb/versions/NFFCYkVT/1.20.1%20OFF%20Backport%201.0.0.mrpack".into(),
            size: 35096,
            sha1: "09ad6dbe2d7a5b19bc0ae8ec097007d5a373f604".into(),
            path: None,
        };
        let events = Channel::new(|_| Ok(()));
        let prepared = prepare_online_archive(info, -2000, &events).await.unwrap();
        assert_eq!(prepared.pack.info().minecraft, "1.20.1");
        assert!(prepared.pack.info().fabric);
        let temporary_path = prepared._directory.path().to_owned();
        let repo = tempfile::tempdir_in(&root).unwrap();
        let client = MinecraftApiClient::new(
            reqwest::Client::new(),
            &crate::io::net::http::api_client::ApiProvider::Official,
        );
        let PreparedPack { pack, _directory } = prepared;
        install_pack(
            pack,
            repo.path(),
            &client,
            "Online Pack",
            vec![],
            -2000,
            &events,
        )
        .await
        .unwrap();
        let instance = repo
            .path()
            .join("versions/Online Pack")
            .canonicalize()
            .unwrap();
        assert!(instance.join("Online Pack.json").is_file());
        assert!(
            instance
                .join("mods/placeholder-api-2.1.3+1.20.1.jar")
                .is_file()
        );
        assert_eq!(
            crate::app::config::ConfigManager::instance()
                .app_state
                .lock()
                .await
                .instance_isolation
                .get(&instance),
            Some(&true)
        );
        assert_eq!(
            fs::canonicalize(crate::app::config::ConfigManager::instance().config_dir()).unwrap(),
            root
        );
        drop(_directory);
        assert!(!temporary_path.exists());
    }
}
