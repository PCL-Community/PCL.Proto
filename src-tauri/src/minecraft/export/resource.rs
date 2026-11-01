use crate::{
    io::download::{DownloadConfig, Downloader, TaskItemReport, TaskStatus},
    minecraft::version::DownloadFile,
};
use anyhow::{Context, ensure};
use std::{path::PathBuf, time::Instant};
use tauri::ipc::Channel;
use tokio::sync::mpsc;

struct TempFile(PathBuf);
impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn validate(info: &DownloadFile) -> anyhow::Result<()> {
    let url = reqwest::Url::parse(&info.url)?;
    let host = url.host_str().unwrap_or_default();
    ensure!(
        url.scheme() == "https" && url.username().is_empty() && url.password().is_none(),
        "Invalid resource URL"
    );
    ensure!(
        ["modrinth.com", "forgecdn.net", "cursecdn.com"]
            .iter()
            .any(|domain| host == *domain || host.ends_with(&format!(".{domain}"))),
        "Unsupported resource host"
    );
    ensure!(
        info.size > 0 && info.size <= 4 * 1024 * 1024 * 1024,
        "Invalid resource size"
    );
    ensure!(
        info.sha1.is_empty() || crate::minecraft::version::valid_sha1(&info.sha1),
        "Invalid resource SHA-1"
    );
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_resource(
    info: DownloadFile,
    destination: PathBuf,
    task_id: i32,
    on_event: Channel<TaskItemReport>,
) -> Result<(), String> {
    async {
        validate(&info)?;
        let parent = destination
            .parent()
            .filter(|p| p.is_absolute() && p.is_dir())
            .context("Invalid destination folder")?;
        ensure!(
            destination.file_name().is_some() && !destination.try_exists()?,
            "Destination already exists or is invalid"
        );
        let temp = TempFile(parent.join(format!(".pcl-resource-{}.tmp", uuid::Uuid::new_v4())));
        let size = info.size;
        let (tx, mut rx) = mpsc::channel(100);
        let downloader = Downloader::default();
        let transfer = downloader.start_resource_download(
            DownloadConfig {
                file_index: 0,
                out_path: temp.0.clone(),
                task_item_id: 0,
                info,
            },
            tx,
        );
        let reports = async {
            let mut last = Instant::now();
            let mut previous = 0;
            while let Some(update) = rx.recv().await {
                let elapsed = last.elapsed();
                if elapsed.as_millis() < 100 && update.progress.status == TaskStatus::Running {
                    continue;
                }
                let bytes = update.progress.downloaded_bytes;
                let _ = on_event.send(TaskItemReport {
                    task_id,
                    item_id: 0,
                    files_remaining: 1,
                    progress: (bytes as f64 / size as f64).min(1.0),
                    status: TaskStatus::Running,
                    speed: Some(
                        bytes.saturating_sub(previous) as f64 / elapsed.as_secs_f64().max(0.001),
                    ),
                });
                last = Instant::now();
                previous = bytes;
            }
        };
        let (downloaded, ()) = tokio::join!(transfer, reports);
        downloaded?;
        std::fs::hard_link(&temp.0, &destination)
            .context("Cannot publish resource (the destination may already exist)")?;
        Ok::<(), anyhow::Error>(())
    }
    .await
    .map_err(|error| format!("{error:#}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validates_resource_metadata_and_hosts() {
        let mut info = DownloadFile {
            url: "https://cdn.modrinth.com/data/a.jar".into(),
            size: 1,
            sha1: String::new(),
            path: None,
        };
        assert!(validate(&info).is_ok());
        info.url = "https://cdn.modrinth.com.example.org/a".into();
        assert!(validate(&info).is_err());
        info.url = "https://edge.forgecdn.net/files/a".into();
        info.sha1 = "invalid".into();
        assert!(validate(&info).is_err());
    }

    #[test]
    fn ipc_accepts_task_manager_argument_names() {
        let app = tauri::test::mock_builder()
            .invoke_handler(tauri::generate_handler![download_resource])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let result = tauri::test::get_ipc_response(
            &window,
            tauri::webview::InvokeRequest {
                cmd: "download_resource".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: "tauri://localhost".parse().unwrap(),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({
                    "info": { "url": "https://example.org/file.jar", "size": 1, "sha1": "", "path": null },
                    "destination": "/tmp/resource-test.jar",
                    "task_id": 1,
                    "on_event": "__CHANNEL__:1"
                })),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );
        assert_eq!(result.unwrap_err(), "Unsupported resource host");
    }
}
