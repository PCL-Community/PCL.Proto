use crate::events::EventSink;
use crate::{
    io::download::{DownloadConfig, Downloader, TaskItemReport, TaskStatus},
    minecraft::version::DownloadFile,
};
use anyhow::{Context, ensure};
use std::path::PathBuf;

struct TempFile(PathBuf);
impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

pub(crate) fn validate(info: &DownloadFile) -> anyhow::Result<()> {
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

pub(crate) async fn download_to(
    downloader: &Downloader,
    info: DownloadFile,
    destination: PathBuf,
    task_id: i32,
    on_event: &EventSink<TaskItemReport>,
) -> anyhow::Result<()> {
    downloader
        .download_with_progress(
            DownloadConfig {
                file_index: 0,
                out_path: destination,
                task_item_id: 0,
                info,
            },
            |bytes, size, speed| {
                on_event(TaskItemReport {
                    task_id,
                    item_id: 0,
                    files_remaining: 1,
                    progress: size
                        .filter(|size| *size > 0)
                        .map_or(0.0, |size| (bytes as f64 / size as f64).min(1.0)),
                    status: TaskStatus::Running,
                    speed: Some(speed),
                });
            },
        )
        .await?;
    Ok(())
}

pub async fn download_resource(
    downloader: &Downloader,
    info: DownloadFile,
    destination: PathBuf,
    task_id: i32,
    on_event: EventSink<TaskItemReport>,
) -> Result<(), String> {
    crate::io::download::tasks::run(task_id, async {
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
            download_to(downloader, info, temp.0.clone(), task_id, &on_event).await?;
            std::fs::hard_link(&temp.0, &destination)
                .context("Cannot publish resource (the destination may already exist)")?;
            Ok::<(), anyhow::Error>(())
        }
        .await
        .map_err(|error| format!("{error:#}"))
    })
    .await
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
}
