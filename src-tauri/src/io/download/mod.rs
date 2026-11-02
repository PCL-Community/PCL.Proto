//! Verified file transfers shared by installation and client/server export.
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
    resources: Arc<DownloadResources>,
    bases: crate::io::net::http::api_client::ApiBases,
}

impl Default for Downloader {
    fn default() -> Self {
        Self::with_bases(crate::io::net::http::api_client::ApiBases::new(
            &crate::io::net::http::api_client::ApiProvider::Official,
        ))
    }
}

#[cfg(test)]
mod http_tests;
mod range;
pub mod resources;

use resources::{DOWNLOAD_RESOURCES, DownloadResources, HttpMode};
use std::sync::{Arc, LazyLock, atomic::Ordering};
use tokio::io::BufWriter;

pub(super) const SMALL_FILE_THRESHOLD: u64 = 4 * 1024 * 1024;
type ProgressSink<'a> = Option<(&'a DownloadConfig, &'a mpsc::Sender<ProgressUpdate>)>;

struct DownloadClients {
    generation: u64,
    multiplexed: reqwest::Client,
    http1: reqwest::Client,
}
static CLIENTS: LazyLock<std::sync::RwLock<DownloadClients>> =
    LazyLock::new(|| std::sync::RwLock::new(build_clients()));

fn build_clients() -> DownloadClients {
    let build = |http1| {
        let mut builder = crate::io::net::proxy::builder()
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(15))
            .pool_max_idle_per_host(64)
            .pool_idle_timeout(Duration::from_secs(90));
        if http1 {
            builder = builder.http1_only();
        }
        builder.build().expect("static download HTTP configuration")
    };
    DownloadClients {
        generation: crate::io::net::proxy::generation(),
        multiplexed: build(false),
        http1: build(true),
    }
}

/// Proxy changes replace both clients outside in-flight requests.
fn clients() -> (reqwest::Client, reqwest::Client) {
    {
        let guard = CLIENTS.read().unwrap();
        if guard.generation == crate::io::net::proxy::generation() {
            return (guard.multiplexed.clone(), guard.http1.clone());
        }
    }
    let mut guard = CLIENTS.write().unwrap();
    if guard.generation != crate::io::net::proxy::generation() {
        *guard = build_clients();
    }
    (guard.multiplexed.clone(), guard.http1.clone())
}

impl Downloader {
    pub(crate) fn source_urls(&self, url: &str) -> Vec<String> {
        self.bases.download_urls(
            url,
            self.resources.settings().file_source,
            self.resources.official_fast.load(Ordering::Relaxed),
        )
    }
    pub fn with_bases(bases: crate::io::net::http::api_client::ApiBases) -> Self {
        Self {
            bases,
            resources: DOWNLOAD_RESOURCES.clone(),
        }
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
                false,
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
    pub async fn download(&self, info: &DownloadFile, path: &Path) -> Result<(), McApiError> {
        self.transfer(info, path, None, true, false)
            .await
            .map(|_| ())
    }
    /// Instance setting "disable file verification": reuse existing files as-is.
    pub async fn download_unverified(
        &self,
        info: &DownloadFile,
        path: &Path,
    ) -> Result<(), McApiError> {
        self.transfer(info, path, None, false, false)
            .await
            .map(|_| ())
    }
    pub async fn start_resource_download(
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
    async fn transfer(
        &self,
        info: &DownloadFile,
        path: &Path,
        progress: ProgressSink<'_>,
        verify: bool,
        allow_missing_hash: bool,
    ) -> Result<u64, McApiError> {
        if !(allow_missing_hash && info.sha1.is_empty())
            && !crate::minecraft::version::valid_sha1(&info.sha1)
        {
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
                    Ok(()) => return Ok(info.size),
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
    ) -> Result<(), McApiError> {
        if info.size >= SMALL_FILE_THRESHOLD && self.resources.settings().file_connections > 1 {
            let partial = partial_file(path);
            match self.download_ranges(info, &partial.0, url, progress).await {
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
                    tokio::fs::rename(&partial.0, path).await?;
                    return Ok(());
                }
                Err(McApiError::RangeUnsupported) => {
                    log::debug!("Range unavailable, using sequential download: {url}");
                    report(progress, 0, TaskStatus::Running).await;
                }
                Err(error) => return Err(error),
            }
        }
        let _lease = self.resources.acquire().await;
        let response = send_request(
            self.resources.settings().http_mode,
            info.size >= SMALL_FILE_THRESHOLD,
            url,
            None,
            None,
        )
        .await?
        .error_for_status()?;
        let partial = partial_file(path);
        let output = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&partial.0)
            .await?;
        let mut output = BufWriter::with_capacity(64 * 1024, output);
        let mut stream = response.bytes_stream();
        let mut hasher = sha1::Sha1::new();
        let mut bytes = 0_u64;
        let mut last_report = tokio::time::Instant::now();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            self.resources.record_bytes(chunk.len());
            bytes += chunk.len() as u64;
            if bytes > info.size {
                return Err(McApiError::InvalidMetadata(format!(
                    "Download exceeds expected size: {}",
                    path.display()
                )));
            }
            self.resources.throttle(chunk.len()).await;
            output.write_all(&chunk).await?;
            hasher.update(&chunk);
            if last_report.elapsed() >= Duration::from_millis(100) {
                report(progress, bytes, TaskStatus::Running).await;
                last_report = tokio::time::Instant::now();
            }
        }
        if bytes != info.size {
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
        tokio::fs::rename(&partial.0, path).await?;
        Ok(())
    }
}
pub(crate) async fn send_request(
    mode: HttpMode,
    large: bool,
    url: &str,
    range: Option<(u64, u64)>,
    timeout: Option<Duration>,
) -> Result<reqwest::Response, reqwest::Error> {
    let send = |client: &reqwest::Client| {
        let mut request = client
            .get(url)
            .header(reqwest::header::ACCEPT_ENCODING, "identity");
        if let Some((start, end)) = range {
            request = request.header(reqwest::header::RANGE, format!("bytes={start}-{end}"));
        }
        if let Some(timeout) = timeout {
            request = request.timeout(timeout);
        }
        request.send()
    };
    let (multiplexed, http1) = clients();
    let client = match mode {
        HttpMode::Http11 => &http1,
        HttpMode::Auto if large => &http1,
        _ => &multiplexed,
    };
    let can_fallback = !std::ptr::eq(client, &http1);
    match send(client).await {
        Err(error) if can_fallback && !error.is_timeout() => {
            log::debug!("HTTP negotiation failed for {url}, retrying HTTP/1.1: {error}");
            send(&http1).await
        }
        result => result,
    }
}
fn partial_file(path: &Path) -> PartialFile {
    PartialFile(path.with_file_name(format!(".pcl-download-{}.part", uuid::Uuid::new_v4())))
}

async fn custom_transfer(
    url: &str,
    destination: &Path,
    on_progress: impl Fn(u64, Option<u64>, f64),
) -> anyhow::Result<()> {
    custom_transfer_checked(url, destination, 0, on_progress).await
}

async fn custom_transfer_checked(
    url: &str,
    destination: &Path,
    min_size: u64,
    on_progress: impl Fn(u64, Option<u64>, f64),
) -> anyhow::Result<()> {
    use anyhow::{Context, ensure};
    crate::utils::external::validate(url).map_err(anyhow::Error::msg)?;
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
    let downloader = Downloader::default();
    let mut last_error = None;
    for attempt in 0..4 {
        if attempt > 0 {
            let _ = tokio::fs::remove_file(&partial.0).await;
            on_progress(0, None, 0.0);
            tokio::time::sleep(Duration::from_millis(300 * attempt)).await;
        }
        match custom_attempt(&downloader, url, &partial.0, &on_progress).await {
            Ok(()) => {
                ensure!(
                    tokio::fs::metadata(&partial.0).await?.len() >= min_size,
                    "Downloaded file is too small"
                );
                std::fs::hard_link(&partial.0, destination)
                    .context("Cannot publish download (the destination may already exist)")?;
                return Ok(());
            }
            Err(error) => {
                if matches!(&error, McApiError::IoError(_)) || permanent_http_error(&error) {
                    return Err(error.into());
                }
                last_error = Some(error);
            }
        }
    }
    Err(last_error.context("Download failed")?.into())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_fabric_installer(
    family: crate::io::net::http::api_client::FabricFamily,
    version: String,
    destination: PathBuf,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<(), String> {
    let url = family
        .installer_url(&version)
        .map_err(|error| error.to_string())?;
    custom_transfer_checked(&url, &destination, 64 * 1024, |bytes, size, speed| {
        let _ = on_event.send(TaskItemReport {
            task_id,
            item_id: 0,
            files_remaining: 1,
            progress: size
                .filter(|size| *size > 0)
                .map_or(0.0, |size| (bytes as f64 / size as f64).min(1.0)),
            status: TaskStatus::Running,
            speed: Some(speed),
        });
    })
    .await
    .map_err(|error| format!("{error:#}"))
}

async fn custom_attempt(
    downloader: &Downloader,
    url: &str,
    partial: &Path,
    on_progress: &impl Fn(u64, Option<u64>, f64),
) -> Result<(), McApiError> {
    let resources = &downloader.resources;
    if resources.settings().file_connections > 1 {
        let size = {
            let _lease = resources.acquire().await;
            let response = send_request(
                resources.settings().http_mode,
                true,
                url,
                Some((0, 0)),
                None,
            )
            .await?;
            if response.status() == reqwest::StatusCode::PARTIAL_CONTENT {
                let size = response
                    .headers()
                    .get(reqwest::header::CONTENT_RANGE)
                    .and_then(|value| value.to_str().ok())
                    .and_then(|value| value.strip_prefix("bytes 0-0/"))
                    .and_then(|value| value.parse::<u64>().ok())
                    .filter(|size| *size > 0);
                if size.is_some()
                    && response.content_length().is_none_or(|length| length == 1)
                    && response
                        .headers()
                        .get(reqwest::header::CONTENT_ENCODING)
                        .is_none_or(|v| v == "identity")
                {
                    let mut stream = response.bytes_stream();
                    let mut bytes = 0;
                    while let Some(chunk) = stream.next().await {
                        let chunk = chunk?;
                        bytes += chunk.len();
                        if bytes > 1 {
                            break;
                        }
                        resources.record_bytes(chunk.len());
                        resources.throttle(chunk.len()).await;
                    }
                    if bytes == 1 { size } else { None }
                } else {
                    None
                }
            } else if response.status().is_success()
                || response.status() == reqwest::StatusCode::RANGE_NOT_SATISFIABLE
            {
                None
            } else {
                response.error_for_status()?;
                None
            }
        };
        if let Some(size) = size.filter(|size| *size >= SMALL_FILE_THRESHOLD) {
            let config = DownloadConfig {
                file_index: 0,
                out_path: partial.to_owned(),
                task_item_id: 0,
                info: DownloadFile {
                    url: url.to_owned(),
                    size,
                    sha1: String::new(),
                    path: None,
                },
            };
            let (tx, mut rx) = mpsc::channel::<ProgressUpdate>(100);
            let reports = async {
                let mut last = std::time::Instant::now();
                let mut previous = 0_u64;
                while let Some(update) = rx.recv().await {
                    let bytes = update.progress.downloaded_bytes.max(previous);
                    if last.elapsed() >= Duration::from_millis(100) {
                        on_progress(
                            bytes,
                            Some(size),
                            bytes.saturating_sub(previous) as f64
                                / last.elapsed().as_secs_f64().max(0.001),
                        );
                        previous = bytes;
                        last = std::time::Instant::now();
                    }
                }
            };
            let (result, ()) = tokio::join!(
                async {
                    let result = downloader
                        .download_ranges_inner(
                            &config.info,
                            partial,
                            url,
                            Some((&config, &tx)),
                            false,
                        )
                        .await;
                    drop(tx);
                    result
                },
                reports
            );
            match result {
                Ok(()) => return Ok(()),
                Err(McApiError::RangeUnsupported) => {
                    let _ = tokio::fs::remove_file(partial).await;
                    on_progress(0, None, 0.0);
                }
                Err(error) => return Err(error),
            }
        }
    }
    let _lease = resources.acquire().await;
    let response = send_request(resources.settings().http_mode, true, url, None, None)
        .await?
        .error_for_status()?;
    let size = response.content_length();
    let mut stream = response.bytes_stream();
    let mut output = BufWriter::new(
        tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(partial)
            .await?,
    );
    let mut bytes = 0_u64;
    let mut previous = 0_u64;
    let mut last = std::time::Instant::now();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        resources.record_bytes(chunk.len());
        resources.throttle(chunk.len()).await;
        output.write_all(&chunk).await?;
        bytes += chunk.len() as u64;
        if last.elapsed() >= Duration::from_millis(100) {
            on_progress(
                bytes,
                size,
                (bytes - previous) as f64 / last.elapsed().as_secs_f64().max(0.001),
            );
            previous = bytes;
            last = std::time::Instant::now();
        }
    }
    if size.is_some_and(|size| size != bytes) {
        return Err(McApiError::InvalidMetadata("Download size mismatch".into()));
    }
    output.flush().await?;
    drop(output);
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_custom_file(
    url: String,
    destination: PathBuf,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<(), String> {
    custom_transfer(&url, &destination, |bytes, size, speed| {
        let _ = on_event.send(TaskItemReport {
            task_id,
            item_id: 0,
            files_remaining: 1,
            progress: size
                .filter(|size| *size > 0)
                .map_or(0.0, |size| (bytes as f64 / size as f64).min(1.0)),
            status: TaskStatus::Running,
            speed: Some(speed),
        });
    })
    .await
    .map_err(|error| format!("{error:#}"))
}

#[tauri::command]
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
