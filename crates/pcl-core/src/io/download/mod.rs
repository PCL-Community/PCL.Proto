//! Verified file transfers shared by installation and client/server export.
use crate::events::EventSink;
use crate::io::net::http::api_client::{McApiError, game::DownloadFile};
use futures_util::StreamExt;
use sha1::Digest;
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use tokio::{io::AsyncWriteExt, sync::mpsc};

#[derive(serde::Serialize, Debug, Clone, Copy)]
pub struct FileProgress {
    pub downloaded_bytes: u64,
    pub status: TaskStatus,
}

#[derive(PartialEq, Debug, serde_repr::Serialize_repr, Default, Clone, Copy)]
#[repr(u8)]
pub enum TaskStatus {
    #[default]
    Pending = 0,
    Running = 1,
    Completed = 2,
    Failed = 3,
}

#[derive(Debug)]
pub struct ProgressUpdate {
    pub file_index: usize,
    pub progress: FileProgress,
    pub item_id: i32,
}

pub struct DownloadConfig {
    pub file_index: usize,
    pub out_path: PathBuf,
    pub task_item_id: i32,
    pub info: DownloadFile,
}

/// Removes incomplete transfers on errors and when their futures are cancelled.
struct PartialFile(PathBuf);
impl Drop for PartialFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

#[derive(Clone)]
pub struct Downloader {
    transport: HttpTransport,
    pub(crate) bases: crate::io::net::http::api_client::ApiBases,
}

impl Default for Downloader {
    fn default() -> Self {
        Self::new(HttpTransport::default())
    }
}

#[cfg(test)]
mod http_tests;
mod range;
pub mod tasks;

use crate::io::net::http::{
    HttpTransport,
    resources::{DownloadResources, SourceKind},
};
use std::sync::{Arc, atomic::Ordering};
use tokio::io::BufWriter;

pub(super) const SMALL_FILE_THRESHOLD: u64 = 4 * 1024 * 1024;
type ProgressSink<'a> = Option<(&'a DownloadConfig, &'a mpsc::Sender<ProgressUpdate>)>;

impl Downloader {
    pub(crate) fn source_urls(&self, url: &str) -> Vec<String> {
        let settings = self.resources().settings();
        self.bases.download_urls(
            url,
            SourceKind::File,
            &settings,
            self.resources().official_fast.load(Ordering::Relaxed),
        )
    }

    pub fn new(transport: HttpTransport) -> Self {
        Self::with_bases(
            transport,
            crate::io::net::http::api_client::ApiBases::new(
                &crate::io::net::http::api_client::ApiProvider::Official,
            ),
        )
    }

    pub fn with_bases(
        transport: HttpTransport,
        bases: crate::io::net::http::api_client::ApiBases,
    ) -> Self {
        Self { transport, bases }
    }

    pub fn transport(&self) -> &HttpTransport {
        &self.transport
    }

    pub fn resources(&self) -> &Arc<DownloadResources> {
        self.transport.resources()
    }

    pub async fn start_download(
        &self,
        options: DownloadConfig,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<(), McApiError> {
        let result = self
            .transfer(
                &options.info,
                &options.out_path,
                Some((&options, &progress_tx)),
                true,
            )
            .await;
        report(
            Some((&options, &progress_tx)),
            result.as_ref().copied().unwrap_or(0),
            if result.is_ok() {
                TaskStatus::Completed
            } else {
                TaskStatus::Failed
            },
        )
        .await;
        result.map(|_| ())
    }
    pub async fn download_with_progress(
        &self,
        options: DownloadConfig,
        mut on_progress: impl FnMut(u64, Option<u64>, f64),
    ) -> Result<(), McApiError> {
        let size = (options.info.size > 0).then_some(options.info.size);
        let (tx, mut rx) = mpsc::channel(100);
        let download = self.start_download(options, tx);
        let report = async {
            let mut last = tokio::time::Instant::now();
            let mut previous = 0;
            while let Some(update) = rx.recv().await {
                if update.progress.status == TaskStatus::Running
                    && last.elapsed() < Duration::from_millis(100)
                {
                    continue;
                }
                let bytes = update.progress.downloaded_bytes.max(previous);
                let elapsed = last.elapsed().as_secs_f64().max(0.001);
                let total =
                    size.or((update.progress.status == TaskStatus::Completed).then_some(bytes));
                on_progress(
                    bytes,
                    total,
                    bytes.saturating_sub(previous) as f64 / elapsed,
                );
                previous = bytes;
                last = tokio::time::Instant::now();
            }
        };
        let (result, ()) = tokio::join!(download, report);
        result
    }
    pub async fn download(&self, info: &DownloadFile, path: &Path) -> Result<(), McApiError> {
        self.transfer(info, path, None, true).await.map(|_| ())
    }
    /// Instance setting "disable file verification": reuse existing files as-is.
    pub async fn download_unverified(
        &self,
        info: &DownloadFile,
        path: &Path,
    ) -> Result<(), McApiError> {
        self.transfer(info, path, None, false).await.map(|_| ())
    }
    /// `size == 0` defers length discovery; an empty SHA-1 selects size-only verification.
    async fn transfer(
        &self,
        info: &DownloadFile,
        path: &Path,
        progress: ProgressSink<'_>,
        verify: bool,
    ) -> Result<u64, McApiError> {
        if !info.sha1.is_empty() && !crate::minecraft::version::valid_sha1(&info.sha1) {
            return Err(McApiError::InvalidMetadata("Invalid download SHA-1".into()));
        }
        let verify = verify && !info.sha1.is_empty();
        report(progress, 0, TaskStatus::Pending).await;
        let cached_path = path.to_owned();
        let expected = info.clone();
        let cached = tokio::task::spawn_blocking(move || {
            if verify {
                std::fs::metadata(&cached_path).is_ok_and(|m| m.len() == expected.size)
                    && crate::io::files::check_sha1(
                        &cached_path,
                        &expected.sha1.to_ascii_lowercase(),
                    )
                    .unwrap_or(false)
            } else {
                cached_path.is_file()
            }
        })
        .await
        .map_err(|e| McApiError::InvalidMetadata(e.to_string()))?;
        if cached {
            return Ok(0);
        }
        let parent = path
            .parent()
            .ok_or_else(|| McApiError::InvalidMetadata("Missing destination parent".into()))?;
        // Complete directory creation before a cancelled task can remove its staging tree.
        std::fs::create_dir_all(parent)?;
        let urls = self.source_urls(&info.url);
        let mut disabled = vec![false; urls.len()];
        let mut last_error = None;
        for attempt in 0..3 {
            for (index, url) in urls.iter().enumerate() {
                if disabled[index] {
                    continue;
                }
                report(progress, 0, TaskStatus::Running).await;
                let result = self.transfer_from(info, path, url, progress, verify).await;
                match result {
                    Ok(size) => return Ok(size),
                    Err(error) => {
                        if matches!(&error, McApiError::IoError(_)) {
                            return Err(error);
                        }
                        disabled[index] = permanent_http_error(&error);
                        log::warn!("Download attempt {} failed for {url}: {error}", attempt + 1);
                        last_error = Some(error);
                    }
                }
            }
            if attempt == 2 || disabled.iter().all(|disabled| *disabled) {
                break;
            }
            tokio::time::sleep(Duration::from_millis(300 * (attempt + 1))).await;
        }
        Err(last_error
            .unwrap_or_else(|| McApiError::InvalidMetadata("No download source available".into())))
    }

    async fn transfer_from(
        &self,
        info: &DownloadFile,
        path: &Path,
        url: &str,
        progress: ProgressSink<'_>,
        verify: bool,
    ) -> Result<u64, McApiError> {
        if self.resources().settings().file_connections > 1
            && (info.size == 0 || info.size >= SMALL_FILE_THRESHOLD)
        {
            let Some(size) = self
                .probe_range(url, (info.size > 0).then_some(info.size))
                .await?
            else {
                return self
                    .transfer_sequential(info, path, url, progress, verify)
                    .await;
            };
            if size < SMALL_FILE_THRESHOLD {
                return self
                    .transfer_sequential(info, path, url, progress, verify)
                    .await;
            }
            let mut ranged = info.clone();
            ranged.size = size;
            let partial = partial_file(path);
            match self
                .download_ranges(&ranged, &partial.0, url, progress)
                .await
            {
                Ok(()) => {
                    if verify {
                        let check_path = partial.0.clone();
                        let hash = info.sha1.to_ascii_lowercase();
                        let valid = tokio::task::spawn_blocking(move || {
                            crate::io::files::check_sha1(&check_path, &hash)
                        })
                        .await
                        .map_err(|e| McApiError::InvalidMetadata(e.to_string()))??;
                        if !valid {
                            return Err(McApiError::Sha1Mismatch(path.to_owned()));
                        }
                    }
                    std::fs::rename(&partial.0, path)?;
                    return Ok(size);
                }
                Err(McApiError::RangeUnsupported) => {
                    log::debug!("Range unavailable, using sequential download: {url}");
                    report(progress, 0, TaskStatus::Running).await;
                }
                Err(error) => return Err(error),
            }
        }
        self.transfer_sequential(info, path, url, progress, verify)
            .await
    }

    async fn transfer_sequential(
        &self,
        info: &DownloadFile,
        path: &Path,
        url: &str,
        progress: ProgressSink<'_>,
        verify: bool,
    ) -> Result<u64, McApiError> {
        let _lease = self.resources().acquire().await;
        let response = self
            .transport
            .send_request(
                self.resources().settings().http_mode,
                info.size == 0 || info.size >= SMALL_FILE_THRESHOLD,
                url,
                None,
                None,
            )
            .await?
            .error_for_status()?;
        let expected = (info.size > 0)
            .then_some(info.size)
            .or(response.content_length());
        let partial = partial_file(path);
        let output = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&partial.0)?;
        let mut output = BufWriter::with_capacity(64 * 1024, tokio::fs::File::from_std(output));
        let mut stream = response.bytes_stream();
        let mut hasher = sha1::Sha1::new();
        let mut bytes = 0_u64;
        let mut last_report = tokio::time::Instant::now();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            self.resources().record_bytes(chunk.len());
            bytes += chunk.len() as u64;
            if expected.is_some_and(|size| bytes > size) {
                return Err(McApiError::InvalidMetadata(format!(
                    "Download exceeds expected size: {}",
                    path.display()
                )));
            }
            self.resources().throttle(chunk.len()).await;
            output.write_all(&chunk).await?;
            hasher.update(&chunk);
            if last_report.elapsed() >= Duration::from_millis(100) {
                report(progress, bytes, TaskStatus::Running).await;
                last_report = tokio::time::Instant::now();
            }
        }
        if expected.is_some_and(|size| bytes != size) {
            return Err(McApiError::InvalidMetadata(format!(
                "Download size mismatch: {}",
                path.display()
            )));
        }
        if verify && !crate::io::files::sha1_hex(hasher.finalize()).eq_ignore_ascii_case(&info.sha1)
        {
            return Err(McApiError::Sha1Mismatch(path.to_owned()));
        }
        output.flush().await?;
        // Download caches are replaceable; avoid an fsync for every tiny asset.
        drop(output);
        std::fs::rename(&partial.0, path)?;
        Ok(bytes)
    }
}
fn partial_file(path: &Path) -> PartialFile {
    PartialFile(path.with_file_name(format!(".pcl-download-{}.part", uuid::Uuid::new_v4())))
}

async fn custom_transfer_checked(
    downloader: &Downloader,
    url: &str,
    destination: &Path,
    min_size: u64,
    on_progress: impl Fn(u64, Option<u64>, f64),
) -> anyhow::Result<()> {
    use anyhow::{Context, ensure};
    let parent = destination
        .parent()
        .filter(|p| p.is_absolute() && p.is_dir())
        .context("Invalid destination folder")?;
    let name = destination
        .file_name()
        .and_then(|n| n.to_str())
        .context("Invalid filename")?;
    crate::io::files::validate_file_name(name)?;
    ensure!(!destination.try_exists()?, "Destination already exists");
    let partial = PartialFile(parent.join(format!(".pcl-custom-{}.part", uuid::Uuid::new_v4())));
    downloader
        .download_with_progress(
            DownloadConfig {
                file_index: 0,
                out_path: partial.0.clone(),
                task_item_id: 0,
                info: DownloadFile {
                    url: url.to_owned(),
                    size: 0,
                    sha1: String::new(),
                    path: None,
                },
            },
            on_progress,
        )
        .await?;
    ensure!(
        tokio::fs::metadata(&partial.0).await?.len() >= min_size,
        "Downloaded file is too small"
    );
    std::fs::hard_link(&partial.0, destination)
        .context("Cannot publish download (the destination may already exist)")?;
    Ok(())
}

async fn run_custom_download(
    downloader: &Downloader,
    url: String,
    destination: PathBuf,
    min_size: u64,
    task_id: i32,
    on_event: EventSink<TaskItemReport>,
) -> Result<(), String> {
    tasks::run(task_id, async {
        custom_transfer_checked(
            downloader,
            &url,
            &destination,
            min_size,
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
        .await
        .map_err(|error| format!("{error:#}"))
    })
    .await
}

pub async fn download_fabric_installer(
    downloader: &Downloader,
    family: crate::io::net::http::api_client::FabricFamily,
    version: String,
    destination: PathBuf,
    task_id: i32,
    on_event: EventSink<TaskItemReport>,
) -> Result<(), String> {
    let url = family
        .installer_url(&version)
        .map_err(|error| error.to_string())?;
    run_custom_download(downloader, url, destination, 64 * 1024, task_id, on_event).await
}

pub async fn download_custom_file(
    downloader: &Downloader,
    url: String,
    destination: PathBuf,
    task_id: i32,
    on_event: EventSink<TaskItemReport>,
) -> Result<(), String> {
    run_custom_download(downloader, url, destination, 0, task_id, on_event).await
}

pub async fn open_custom_download_folder(directory: PathBuf) -> Result<(), String> {
    if !directory.is_absolute() || !directory.is_dir() {
        return Err("Invalid destination folder".into());
    }
    crate::io::files::open_directory(&directory)
}
fn permanent_http_error(error: &McApiError) -> bool {
    matches!(error, McApiError::RequestError(e) if e.status().is_some_and(|s| s.is_client_error() && s.as_u16() != 408 && s.as_u16() != 429))
}

async fn report(
    progress: Option<(&DownloadConfig, &mpsc::Sender<ProgressUpdate>)>,
    bytes: u64,
    status: TaskStatus,
) {
    if let Some((options, tx)) = progress {
        // UI lifetime does not control whether verified files may finish downloading.
        let _ = tx
            .send(ProgressUpdate {
                file_index: options.file_index,
                item_id: options.task_item_id,
                progress: FileProgress {
                    downloaded_bytes: bytes,
                    status,
                },
            })
            .await;
    }
}

#[derive(serde::Serialize, Debug)]
pub struct TaskItemReport {
    pub task_id: i32,
    pub item_id: i32,
    pub files_remaining: usize,
    pub progress: f64,
    pub status: TaskStatus,
    pub speed: Option<f64>,
}

pub(crate) struct StageProgress {
    pub id: i32,
    sizes: Vec<u64>,
    files: Vec<FileProgress>,
    total: u64,
    downloaded: u64,
    transferred: u64,
    remaining: usize,
    failed: bool,
    last_bytes: u64,
    last_sent: std::time::Instant,
}

impl StageProgress {
    pub fn new(id: i32, sizes: Vec<u64>) -> Self {
        Self {
            id,
            total: sizes.iter().sum(),
            files: vec![
                FileProgress {
                    downloaded_bytes: 0,
                    status: TaskStatus::Pending
                };
                sizes.len()
            ],
            remaining: sizes.len(),
            sizes,
            downloaded: 0,
            transferred: 0,
            failed: false,
            last_bytes: 0,
            last_sent: std::time::Instant::now(),
        }
    }

    pub fn update(&mut self, index: usize, progress: FileProgress) {
        let previous = self.files[index];
        let size = self.sizes[index];
        let contribution = |file: FileProgress| {
            if file.status == TaskStatus::Completed {
                size
            } else {
                file.downloaded_bytes.min(size)
            }
        };
        self.downloaded -= contribution(previous);
        self.downloaded += contribution(progress);
        self.transferred -= previous.downloaded_bytes;
        self.transferred += progress.downloaded_bytes.min(size);
        if previous.status != TaskStatus::Completed && progress.status == TaskStatus::Completed {
            self.remaining -= 1;
        }
        self.failed |= progress.status == TaskStatus::Failed;
        self.files[index] = FileProgress {
            downloaded_bytes: progress.downloaded_bytes.min(self.sizes[index]),
            ..progress
        };
    }

    pub fn report(&mut self, task_id: i32) -> TaskItemReport {
        let status = if self.failed {
            TaskStatus::Failed
        } else if self.remaining == 0 {
            TaskStatus::Completed
        } else {
            TaskStatus::Running
        };
        let progress = if self.remaining == 0 {
            1.0
        } else if self.total == 0 {
            (self.files.len() - self.remaining) as f64 / self.files.len() as f64
        } else {
            (self.downloaded as f64 / self.total as f64).clamp(0.0, 1.0)
        };
        let elapsed = self.last_sent.elapsed().as_secs_f64().max(0.001);
        let speed = if status == TaskStatus::Running {
            self.transferred.saturating_sub(self.last_bytes) as f64 / elapsed
        } else {
            0.0
        };
        self.last_bytes = self.transferred;
        self.last_sent = std::time::Instant::now();
        TaskItemReport {
            task_id,
            item_id: self.id,
            files_remaining: self.remaining,
            progress,
            status,
            speed: Some(speed),
        }
    }

    pub fn should_report(&self) -> bool {
        self.last_sent.elapsed() >= Duration::from_millis(100) || self.remaining == 0 || self.failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_handles_empty_stages_retries_and_failures() {
        let mut empty = StageProgress::new(1, vec![]);
        assert_eq!(empty.report(0).progress, 1.0);
        let mut stage = StageProgress::new(1, vec![10, 10]);
        stage.update(
            0,
            FileProgress {
                downloaded_bytes: 8,
                status: TaskStatus::Running,
            },
        );
        assert_eq!(stage.report(0).progress, 0.4);
        stage.update(
            0,
            FileProgress {
                downloaded_bytes: 0,
                status: TaskStatus::Running,
            },
        );
        assert!(stage.report(0).speed.unwrap().is_finite());
        stage.update(
            0,
            FileProgress {
                downloaded_bytes: 0,
                status: TaskStatus::Failed,
            },
        );
        stage.update(
            1,
            FileProgress {
                downloaded_bytes: 10,
                status: TaskStatus::Completed,
            },
        );
        assert_eq!(stage.report(0).status, TaskStatus::Failed);
        let mut zero = StageProgress::new(1, vec![0]);
        assert!(zero.report(0).progress.is_finite());
        zero.update(
            0,
            FileProgress {
                downloaded_bytes: 0,
                status: TaskStatus::Completed,
            },
        );
        assert_eq!(zero.report(0).progress, 1.0);
    }
    #[test]
    fn cached_files_advance_progress_without_inflating_network_speed() {
        let mut stage = StageProgress::new(1, vec![10, 10]);
        stage.report(0);
        stage.update(
            0,
            FileProgress {
                downloaded_bytes: 0,
                status: TaskStatus::Completed,
            },
        );
        assert!(
            !stage.should_report(),
            "individual files do not bypass stage throttling"
        );
        let report = stage.report(0);
        assert_eq!(report.progress, 0.5);
        assert_eq!(report.speed, Some(0.0));
        stage.update(
            1,
            FileProgress {
                downloaded_bytes: 0,
                status: TaskStatus::Completed,
            },
        );
        assert!(stage.should_report(), "stage completion is always reported");
        assert_eq!(stage.report(0).progress, 1.0);
    }
}
