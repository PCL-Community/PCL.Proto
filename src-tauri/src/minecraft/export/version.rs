use crate::{
    io::download::{DownloadConfig, Downloader, TaskItemReport, TaskStatus},
    io::net::http::api_client::{MinecraftApiClient, game::DownloadFile},
};
use anyhow::{Context, bail, ensure};
use std::{
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use tauri::{State, ipc::Channel};
use tokio::sync::mpsc;

// Only directories created by this operation are removed when a download fails.
struct ExportDirectory {
    path: PathBuf,
    keep: bool,
}

impl ExportDirectory {
    fn create(path: PathBuf) -> anyhow::Result<Self> {
        fs::create_dir(&path)
            .with_context(|| format!("Cannot create {} (it may already exist)", path.display()))?;
        Ok(Self { path, keep: false })
    }
}

impl Drop for ExportDirectory {
    fn drop(&mut self) {
        if self.keep {
            return;
        }
        if let Err(error) = fs::remove_dir_all(&self.path) {
            log::warn!(
                "Cannot clean export directory {}: {error}",
                self.path.display()
            );
        }
    }
}

fn validate_version_id(id: &str) -> anyhow::Result<()> {
    ensure!(
        !id.is_empty()
            && ![".", ".."].contains(&id)
            && !id.ends_with(['.', ' '])
            && id
                .chars()
                .all(|c| c.is_alphanumeric() || " ._-+∞".contains(c)),
        "Invalid Minecraft version ID"
    );
    Ok(())
}

fn download_info(
    metadata: &serde_json::Value,
    id: &str,
    server: bool,
) -> anyhow::Result<DownloadFile> {
    ensure!(
        metadata["id"].as_str() == Some(id),
        "Version metadata ID mismatch"
    );
    let kind = if server { "server" } else { "client" };
    let value = &metadata["downloads"][kind];
    if value.is_null() {
        if server {
            bail!("Minecraft.Download.Error.NoOfficialServerDownload");
        }
        bail!("No official {kind} download is available for Minecraft {id}");
    }
    let mut info: DownloadFile = serde_json::from_value(value.clone())?;
    ensure!(
        info.size > 0 && info.sha1.len() == 40 && info.sha1.bytes().all(|c| c.is_ascii_hexdigit()),
        "Invalid download size or SHA-1"
    );
    ensure!(
        reqwest::Url::parse(&info.url)?.scheme() == "https",
        "Download URL must use HTTPS"
    );
    // Paths in remote metadata never select local export destinations.
    info.path = None;
    Ok(info)
}

async fn download_jar(
    info: DownloadFile,
    path: PathBuf,
    task_id: i32,
    on_event: &Channel<TaskItemReport>,
    downloader: &Downloader,
) -> anyhow::Result<()> {
    let size = info.size;
    let (tx, mut rx) = mpsc::channel(100);
    let download = downloader.start_download(
        DownloadConfig {
            file_index: 0,
            task_item_id: 1,
            out_path: path.clone(),
            info,
        },
        tx,
    );
    let report = async {
        let mut last = Instant::now();
        let mut previous_bytes = 0;
        while let Some(update) = rx.recv().await {
            let elapsed = last.elapsed();
            if elapsed.as_millis() < 100 && update.progress.status == TaskStatus::Running {
                continue;
            }
            let bytes = update.progress.downloaded_bytes;
            on_event.send(TaskItemReport {
                task_id,
                item_id: 1,
                files_remaining: 1,
                progress: (bytes as f64 / size as f64).min(1.0),
                // Publication of the verified files is the final completion step.
                status: TaskStatus::Running,
                speed: Some(
                    bytes.saturating_sub(previous_bytes) as f64 / elapsed.as_secs_f64().max(0.001),
                ),
            })?;
            last = Instant::now();
            previous_bytes = bytes;
        }
        anyhow::Ok(())
    };
    let (downloaded, reported) = tokio::join!(download, report);
    downloaded?;
    reported?;
    ensure!(
        fs::metadata(path)?.len() == size,
        "Downloaded file size mismatch"
    );
    Ok(())
}

fn publish(staging: &Path, destination: PathBuf) -> anyhow::Result<PathBuf> {
    let mut directory = ExportDirectory::create(destination)?;
    for entry in fs::read_dir(staging)? {
        let entry = entry?;
        fs::rename(entry.path(), directory.path.join(entry.file_name()))?;
    }
    let path = directory.path.clone();
    // Keep the completed directory; the staging directory still cleans itself up.
    directory.keep = true;
    Ok(path)
}

async fn export_version(
    client: &MinecraftApiClient,
    version_id: &str,
    directory: &Path,
    server: bool,
    task_id: i32,
    on_event: Channel<TaskItemReport>,
) -> anyhow::Result<String> {
    validate_version_id(version_id)?;
    ensure!(
        directory.is_absolute() && directory.is_dir(),
        "Select an existing absolute directory"
    );
    let directory = directory.canonicalize()?;
    let destination = directory.join(if server {
        format!("{version_id}-server")
    } else {
        version_id.to_owned()
    });
    ensure!(
        !destination.try_exists()?,
        "Destination already exists: {}",
        destination.display()
    );
    let metadata = client.get_version_metadata(version_id).await?;
    let info = download_info(&metadata, version_id, server)?;
    let staging =
        ExportDirectory::create(directory.join(format!(".pcl-export-{}", uuid::Uuid::new_v4())))?;
    on_event.send(TaskItemReport {
        task_id,
        item_id: 0,
        files_remaining: 0,
        progress: 1.0,
        status: TaskStatus::Completed,
        speed: None,
    })?;
    let jar_name = if server {
        format!("{version_id}-server.jar")
    } else {
        format!("{version_id}.jar")
    };
    let downloader = Downloader::with_bases(client.api_bases_async().await);
    download_jar(
        info,
        staging.path.join(&jar_name),
        task_id,
        &on_event,
        &downloader,
    )
    .await?;
    if server {
        fs::write(
            staging.path.join("Launch Server.bat"),
            format!(
                "@echo off\r\ncd /d \"%~dp0\"\r\njava -Xmx4G -Xms1G -jar \"{jar_name}\" nogui\r\npause\r\n"
            ),
        )?;
        fs::write(
            staging.path.join("Launch Server.sh"),
            format!(
                "#!/bin/sh\ncd -- \"$(dirname -- \"$0\")\" || exit 1\nexec java -Xmx4G -Xms1G -jar '{jar_name}' nogui\n"
            ),
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(
                staging.path.join("Launch Server.sh"),
                fs::Permissions::from_mode(0o755),
            )?;
        }
    } else {
        fs::write(
            staging.path.join(format!("{version_id}.json")),
            serde_json::to_vec_pretty(&metadata)?,
        )?;
    }
    let path = publish(&staging.path, destination)?;
    // A closed UI channel must not turn an already published export into a failure.
    let _ = on_event.send(TaskItemReport {
        task_id,
        item_id: 1,
        files_remaining: 0,
        progress: 1.0,
        status: TaskStatus::Completed,
        speed: None,
    });
    Ok(path.to_string_lossy().into_owned())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_minecraft_version(
    client: State<'_, &MinecraftApiClient>,
    version_id: String,
    directory: PathBuf,
    server: bool,
    task_id: i32,
    on_event: Channel<TaskItemReport>,
) -> Result<String, String> {
    crate::io::download::tasks::run(task_id, async {
        export_version(&client, &version_id, &directory, server, task_id, on_event)
            .await
            .map_err(|error| error.to_string())
    })
    .await
}

#[tauri::command]
pub async fn open_minecraft_wiki(url: String) -> Result<(), String> {
    let parsed = reqwest::Url::parse(&url).map_err(|error| error.to_string())?;
    if parsed.scheme() != "https"
        || !parsed
            .host_str()
            .is_some_and(|host| host == "minecraft.wiki" || host.ends_with(".minecraft.wiki"))
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port().is_some()
    {
        return Err("Invalid Minecraft Wiki URL".into());
    }
    tauri::async_runtime::spawn_blocking(move || crate::utils::external::open(parsed.as_str()))
        .await
        .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha1::Digest;
    use std::io::{Read, Write};

    fn temp_directory() -> ExportDirectory {
        ExportDirectory::create(
            std::env::temp_dir().join(format!("pcl-export-test-{}", uuid::Uuid::new_v4())),
        )
        .unwrap()
    }

    #[tokio::test]
    async fn refuses_external_or_unsafe_wiki_urls() {
        for url in [
            "http://minecraft.wiki/w/Test",
            "https://minecraft.wiki.evil.test",
            "https://minecraft.wiki:8443/w/Test",
            "https://user:pass@minecraft.wiki",
            "file:///tmp/test",
        ] {
            assert!(open_minecraft_wiki(url.into()).await.is_err(), "{url}");
        }
    }

    #[tokio::test]
    #[ignore = "Downloads official Minecraft files; run explicitly for network smoke testing"]
    async fn official_client_and_server_export() {
        let client = MinecraftApiClient::new(
            reqwest::Client::new(),
            &crate::io::net::http::api_client::ApiProvider::Official,
        );
        let directory = temp_directory();
        for server in [false, true] {
            let path = export_version(
                &client,
                "1.2.5",
                &directory.path,
                server,
                0,
                Channel::new(|_| Ok(())),
            )
            .await
            .unwrap();
            let path = Path::new(&path);
            if server {
                assert!(path.join("1.2.5-server.jar").is_file());
                assert!(path.join("Launch Server.bat").is_file());
                assert!(path.join("Launch Server.sh").is_file());
            } else {
                assert!(path.join("1.2.5.jar").is_file());
                let metadata: serde_json::Value =
                    serde_json::from_slice(&fs::read(path.join("1.2.5.json")).unwrap()).unwrap();
                assert_eq!(metadata["id"], "1.2.5");
            }
        }
        let error = export_version(
            &client,
            "rd-132211",
            &directory.path,
            true,
            0,
            Channel::new(|_| Ok(())),
        )
        .await
        .unwrap_err();
        assert!(error.to_string().contains("NoOfficialServerDownload"));
        assert!(!directory.path.join("rd-132211-server").exists());
    }

    #[test]
    fn validates_paths_and_missing_server_without_rejecting_old_clients() {
        for id in [
            "",
            ".",
            "..",
            "../escape",
            "a/b",
            "a\\b",
            "a%PATH%",
            "a'",
            "a\n",
            "a.",
        ] {
            assert!(validate_version_id(id).is_err(), "{id:?}");
        }
        for id in ["1.21.1", "b1.7.3", "3D Shareware v1.34", "20w14∞"] {
            assert!(validate_version_id(id).is_ok());
        }
        let metadata = serde_json::json!({
            "id": "b1.7.3",
            "downloads": {"client": {"url": "https://piston-data.mojang.com/client.jar", "size": 1, "sha1": "a".repeat(40), "path": "../../escape"}}
        });
        assert!(
            download_info(&metadata, "b1.7.3", false)
                .unwrap()
                .path
                .is_none()
        );
        assert!(
            download_info(&metadata, "b1.7.3", true)
                .unwrap_err()
                .to_string()
                .contains("NoOfficialServerDownload")
        );
        assert!(download_info(&metadata, "wrong", false).is_err());
        let mut invalid = metadata.clone();
        invalid["downloads"]["client"]["sha1"] = "bad".into();
        assert!(download_info(&invalid, "b1.7.3", false).is_err());
    }

    #[test]
    fn publication_preserves_existing_directories_and_cleans_unfinished_files() {
        let root = temp_directory();
        let staging = ExportDirectory::create(root.path.join("staging")).unwrap();
        fs::write(staging.path.join("client.jar"), "verified").unwrap();
        let destination = root.path.join("version");
        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("world.dat"), "keep").unwrap();
        assert!(publish(&staging.path, destination.clone()).is_err());
        assert_eq!(
            fs::read_to_string(destination.join("world.dat")).unwrap(),
            "keep"
        );
        let published = publish(&staging.path, root.path.join("new-version")).unwrap();
        assert_eq!(
            fs::read_to_string(published.join("client.jar")).unwrap(),
            "verified"
        );
        let staging_path = staging.path.clone();
        drop(staging);
        assert!(!staging_path.exists());
        assert!(published.exists());
    }

    // Real HTTP responses exercise the shared downloader, digest checks, and IPC relay.
    async fn checked_http_download(
        status: &str,
        body: &[u8],
        expected_sha1: String,
        requests: usize,
    ) -> anyhow::Result<()> {
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        let url = format!("http://{}/test.jar", listener.local_addr()?);
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
            body.len()
        )
        .into_bytes();
        let body = body.to_vec();
        let size = body.len() as u64;
        let server = std::thread::spawn(move || {
            listener.set_nonblocking(true).unwrap();
            let deadline = Instant::now() + std::time::Duration::from_secs(5);
            let mut served = 0;
            while served < requests && Instant::now() < deadline {
                if let Ok((mut stream, _)) = listener.accept() {
                    stream
                        .set_read_timeout(Some(std::time::Duration::from_secs(1)))
                        .unwrap();
                    let mut request = [0u8; 4096];
                    stream.read(&mut request).unwrap();
                    stream.write_all(&response).unwrap();
                    stream.write_all(&body).unwrap();
                    served += 1;
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(5));
                }
            }
            assert_eq!(served, requests);
        });
        let directory = temp_directory();
        let result = download_jar(
            DownloadFile {
                sha1: expected_sha1,
                size,
                url,
                path: None,
            },
            directory.path.join("test.jar"),
            7,
            &Channel::new(|_| Ok(())),
            &Downloader::default(),
        )
        .await;
        server.join().unwrap();
        if result.is_ok() {
            assert!(directory.path.join("test.jar").is_file());
        }
        result
    }

    #[tokio::test]
    async fn downloads_only_verified_files_and_rejects_http_errors() {
        let digest = crate::io::files::sha1_hex(sha1::Sha1::digest(b"jar"));
        checked_http_download("200 OK", b"jar", digest, 1)
            .await
            .unwrap();
        assert!(
            checked_http_download("200 OK", b"jar", "0".repeat(40), 3)
                .await
                .is_err()
        );
        assert!(
            checked_http_download("404 Not Found", b"missing", "0".repeat(40), 1)
                .await
                .is_err()
        );
    }
}
