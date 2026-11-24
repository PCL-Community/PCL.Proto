//! aria2-rust based verified file transfers shared by installation and export.

use crate::events::EventSink;
use crate::io::net::http::api_client::{McApiError, game::DownloadFile};
use aria2_core::{
    DownloadEngine, DownloadManager as AriaDownloadManager, DownloadStatus, RequestGroupMan,
};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Duration,
};
use tokio::sync::{Mutex as AsyncMutex, mpsc};

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
    pub total_bytes: Option<u64>,
}

pub struct DownloadConfig {
    pub file_index: usize,
    pub out_path: PathBuf,
    pub task_item_id: i32,
    pub info: DownloadFile,
}

#[derive(Clone)]
pub struct DownloadManager {
    transport: HttpTransport,
    sources: Arc<SourceResolver>,
    engine: Arc<AsyncMutex<Option<AriaDownloadManager>>>,
    telemetry: Arc<DownloadTelemetry>,
}

#[derive(Default)]
struct DownloadTelemetry {
    tasks: Mutex<HashMap<u64, TaskTelemetry>>,
    downloaded_bytes: AtomicU64,
}

#[derive(Clone, Copy)]
struct TaskTelemetry {
    connections: u32,
    speed: u64,
}

struct TelemetryLease {
    telemetry: Arc<DownloadTelemetry>,
    gid: u64,
}

impl Drop for TelemetryLease {
    fn drop(&mut self) {
        if let Ok(mut tasks) = self.telemetry.tasks.lock() {
            tasks.remove(&self.gid);
        }
    }
}

pub mod tasks;

use crate::io::net::http::{
    HttpTransport, RequestTarget,
    resources::{DownloadResources, DownloadStats, HttpMode, SourceKind},
};

type SourceResolver = dyn Fn(&str, SourceKind) -> Vec<RequestTarget> + Send + Sync;
type ProgressSink<'a> = Option<(&'a DownloadConfig, &'a mpsc::Sender<ProgressUpdate>)>;

impl Default for DownloadManager {
    fn default() -> Self {
        Self::new(HttpTransport::default())
    }
}

impl DownloadManager {
    pub(crate) fn resolve_sources(&self, url: &str, kind: SourceKind) -> Vec<RequestTarget> {
        (self.sources)(url, kind)
    }

    pub fn new(transport: HttpTransport) -> Self {
        Self {
            telemetry: Arc::new(DownloadTelemetry::default()),
            transport,
            sources: Arc::new(|url, _| vec![RequestTarget::new(url)]),
            engine: Arc::new(AsyncMutex::new(None)),
        }
    }

    /// Supply ordered destinations with their headers to aria2-rust.
    pub fn with_sources(
        mut self,
        resolver: impl Fn(&str, SourceKind) -> Vec<RequestTarget> + Send + Sync + 'static,
    ) -> Self {
        self.sources = Arc::new(resolver);
        self
    }

    pub fn transport(&self) -> &HttpTransport {
        &self.transport
    }

    pub fn resources(&self) -> &Arc<DownloadResources> {
        self.transport.resources()
    }

    pub fn stats(&self) -> DownloadStats {
        let (active_connections, speed) = self
            .telemetry
            .tasks
            .lock()
            .map(|tasks| {
                tasks
                    .values()
                    .fold((0usize, 0f64), |(connections, speed), task| {
                        (
                            connections + task.connections as usize,
                            speed + task.speed as f64,
                        )
                    })
            })
            .unwrap_or_default();
        DownloadStats {
            active_connections,
            max_connections: self.resources().settings().max_connections,
            speed,
            downloaded_bytes: self.telemetry.downloaded_bytes.load(Ordering::Relaxed),
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
            )
            .await;
        let completed_bytes = result.as_ref().copied().unwrap_or(0);
        report(
            Some((&options, &progress_tx)),
            completed_bytes,
            (options.info.size > 0)
                .then_some(options.info.size)
                .or_else(|| result.is_ok().then_some(completed_bytes)),
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
                let total = size.or(update.total_bytes);
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
        report(
            progress,
            0,
            (info.size > 0).then_some(info.size),
            TaskStatus::Pending,
        )
        .await;

        if self.is_cached(info, path, verify).await? {
            return Ok(0);
        }

        let parent = path
            .parent()
            .ok_or_else(|| McApiError::InvalidMetadata("Missing destination parent".into()))?;
        std::fs::create_dir_all(parent)?;
        let targets = (self.sources)(&info.url, SourceKind::File);
        if targets.is_empty() {
            return Err(McApiError::InvalidMetadata(
                "No download source available".into(),
            ));
        }

        let staging = staging_path(path);
        let manager = self.ensure_engine().await?;
        self.configure_engine(&manager)?;
        let source_groups = targets_to_aria_sources(targets);
        let mut last_error = None;
        for attempt in 0..3 {
            let mut result = None;
            for source_group in &source_groups {
                let options = self.download_options(&staging, source_group.headers.clone(), info);
                let handle = manager
                    .add_uri(source_group.uris.clone(), options)
                    .map_err(|error| McApiError::DownloadError(error.to_string()))?;
                let gid = handle.gid().value();
                let _telemetry = TelemetryLease {
                    telemetry: self.telemetry.clone(),
                    gid,
                };
                if let Some(task_id) = tasks::current_task_id() {
                    tasks::register_abort(task_id, handle.clone())
                        .map_err(McApiError::DownloadError)?;
                }

                let transfer = self.wait_for_transfer(&handle, info.size, progress).await;
                if let Some(task_id) = tasks::current_task_id() {
                    tasks::clear_abort(task_id);
                }
                match transfer {
                    Ok(bytes) => {
                        result = Some(bytes);
                        break;
                    }
                    Err(error) => last_error = Some(error),
                }
            }
            let result = match result {
                Some(result) => result,
                None if attempt < 2 => continue,
                None => {
                    return Err(last_error.unwrap_or_else(|| {
                        McApiError::DownloadError("All download sources failed".into())
                    }));
                }
            };
            let bytes = std::fs::metadata(&staging)?.len();
            if info.size > 0 && bytes != info.size {
                let error = McApiError::InvalidMetadata(format!(
                    "Download size mismatch: {}",
                    path.display()
                ));
                if attempt < 2 {
                    last_error = Some(error);
                    reset_staging_for_retry(&staging);
                    continue;
                }
                return Err(error);
            }
            if verify {
                let check_path = staging.clone();
                let hash = info.sha1.to_ascii_lowercase();
                let valid = tokio::task::spawn_blocking(move || {
                    crate::io::files::check_sha1(&check_path, &hash)
                })
                .await
                .map_err(|error| McApiError::InvalidMetadata(error.to_string()))??;
                if !valid {
                    let error = McApiError::Sha1Mismatch(path.to_owned());
                    if attempt < 2 {
                        last_error = Some(error);
                        reset_staging_for_retry(&staging);
                        continue;
                    }
                    return Err(error);
                }
            }
            std::fs::rename(&staging, path)?;
            remove_control_file(&staging);
            return Ok(result.max(bytes));
        }
        Err(last_error.unwrap_or_else(|| McApiError::InvalidMetadata("Download failed".into())))
    }

    async fn is_cached(
        &self,
        info: &DownloadFile,
        path: &Path,
        verify: bool,
    ) -> Result<bool, McApiError> {
        let path = path.to_owned();
        let expected = info.clone();
        tokio::task::spawn_blocking(move || {
            if verify {
                std::fs::metadata(&path).is_ok_and(|metadata| {
                    (expected.size == 0 || metadata.len() == expected.size)
                        && crate::io::files::check_sha1(&path, &expected.sha1.to_ascii_lowercase())
                            .unwrap_or(false)
                })
            } else {
                path.is_file()
            }
        })
        .await
        .map_err(|error| McApiError::InvalidMetadata(error.to_string()))
    }

    async fn ensure_engine(&self) -> Result<AriaDownloadManager, McApiError> {
        let mut engine = self.engine.lock().await;
        if let Some(manager) = engine.as_ref() {
            return Ok(manager.clone());
        }
        let group_man = Arc::new(RequestGroupMan::new());
        let mut download_engine = DownloadEngine::new();
        download_engine.set_keep_alive(true);
        let handle = download_engine
            .start_with_request_group_man(group_man)
            .map_err(|error| McApiError::DownloadError(error.to_string()))?;
        let manager = handle.downloads().clone();
        tokio::spawn(async move {
            if let Err(error) = handle.wait().await {
                log::error!("aria2-rust download engine stopped: {error}");
            }
        });
        *engine = Some(manager.clone());
        Ok(manager)
    }

    fn configure_engine(&self, manager: &AriaDownloadManager) -> Result<(), McApiError> {
        let settings = self.resources().settings();
        manager
            .set_max_concurrent(settings.max_connections as u32)
            .map_err(|error| McApiError::DownloadError(error.to_string()))?;
        manager
            .set_global_rate_limit(
                (settings.speed_limit > 0).then_some(settings.speed_limit),
                None,
            )
            .map_err(|error| McApiError::DownloadError(error.to_string()))?;
        Ok(())
    }

    fn download_options(
        &self,
        staging: &Path,
        headers: Vec<String>,
        info: &DownloadFile,
    ) -> aria2_core::DownloadOptions {
        let settings = self.resources().settings();
        aria2_core::DownloadOptions {
            split: Some(if info.size > 0 && info.size < 4 * 1024 * 1024 {
                1
            } else {
                settings.file_connections as u64
            }),
            max_connection_per_server: Some(if info.size > 0 && info.size < 4 * 1024 * 1024 {
                1
            } else {
                settings.file_connections as u16
            }),
            max_download_limit: (settings.speed_limit > 0).then_some(settings.speed_limit),
            dir: staging
                .parent()
                .map(|path| path.to_string_lossy().into_owned()),
            out: staging
                .file_name()
                .map(|name| name.to_string_lossy().into_owned()),
            continue_download: true,
            allow_overwrite: true,
            auto_file_renaming: false,
            always_resume: true,
            max_retries: 3,
            retry_wait: 1,
            user_agent: Some(self.transport.user_agent()),
            http_version: match settings.http_mode {
                HttpMode::Auto => aria2_core::http::HttpVersion::Auto,
                HttpMode::Http11 => aria2_core::http::HttpVersion::Http11,
                HttpMode::Http2 => aria2_core::http::HttpVersion::Http2,
            },
            header: headers,
            ..Default::default()
        }
    }

    async fn wait_for_transfer(
        &self,
        handle: &aria2_core::DownloadHandle,
        expected_size: u64,
        progress: ProgressSink<'_>,
    ) -> Result<u64, McApiError> {
        let gid = handle.gid().value();
        let mut wait = Box::pin(handle.wait());
        let mut tick = tokio::time::interval(Duration::from_millis(100));
        let mut previous_bytes = 0;
        loop {
            tokio::select! {
                result = &mut wait => {
                    let result = result.map_err(|error| McApiError::DownloadError(error.to_string()))?;
                    let bytes = result.completed_length;
                    self.telemetry.downloaded_bytes.fetch_add(
                        bytes.saturating_sub(previous_bytes),
                        Ordering::Relaxed,
                    );
                    if matches!(result.status, DownloadStatus::Complete) {
                        report(
                            progress,
                            bytes,
                            (result.total_length > 0)
                                .then_some(result.total_length)
                                .or((expected_size > 0).then_some(expected_size)),
                            TaskStatus::Completed,
                        )
                        .await;
                        return Ok(bytes);
                    }
                    return Err(McApiError::DownloadError(result.message));
                }
                _ = tick.tick() => {
                    if let Some(snapshot) = handle.status_snapshot() {
                        let bytes = snapshot.completed_length;
                        self.telemetry.tasks.lock().map(|mut tasks| {
                            tasks.insert(gid, TaskTelemetry {
                                connections: snapshot.connections,
                                speed: snapshot.download_speed,
                            });
                        }).ok();
                        self.telemetry.downloaded_bytes.fetch_add(
                            bytes.saturating_sub(previous_bytes),
                            Ordering::Relaxed,
                        );
                        previous_bytes = bytes;
                        report(
                            progress,
                            bytes,
                            (snapshot.total_length.max(expected_size) > 0)
                                .then_some(snapshot.total_length.max(expected_size)),
                            TaskStatus::Running,
                        )
                        .await;
                    }
                }
            }
        }
    }
}

struct AriaSourceGroup {
    uris: Vec<String>,
    headers: Vec<String>,
}

fn targets_to_aria_sources(targets: Vec<RequestTarget>) -> Vec<AriaSourceGroup> {
    let mut groups = Vec::new();
    for target in targets {
        let mut headers = target
            .headers
            .iter()
            .filter_map(|(name, value)| {
                Some(format!("{}: {}", name.as_str(), value.to_str().ok()?))
            })
            .collect::<Vec<_>>();
        headers.sort_unstable();
        if let Some(group) = groups
            .iter_mut()
            .find(|group: &&mut AriaSourceGroup| group.headers == headers)
        {
            group.uris.push(target.url);
        } else {
            groups.push(AriaSourceGroup {
                uris: vec![target.url],
                headers,
            });
        }
    }
    groups
}

fn staging_path(path: &Path) -> PathBuf {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("download");
    path.with_file_name(format!(".pcl-download-{name}.part"))
}

fn remove_control_file(staging: &Path) {
    let control = PathBuf::from(format!("{}.aria2", staging.display()));
    let _ = std::fs::remove_file(control);
}

fn reset_staging_for_retry(staging: &Path) {
    let _ = std::fs::remove_file(staging);
    remove_control_file(staging);
}

pub async fn download_fabric_installer(
    manager: &DownloadManager,
    family: crate::minecraft::mod_loader::FabricFamily,
    version: String,
    destination: PathBuf,
    task_id: i32,
    on_event: EventSink<TaskItemReport>,
) -> Result<(), String> {
    let url = family
        .installer_url(&version)
        .map_err(|error| error.to_string())?;
    run_custom_download(manager, url, destination, 64 * 1024, task_id, on_event).await
}

pub async fn download_custom_file(
    manager: &DownloadManager,
    url: String,
    destination: PathBuf,
    task_id: i32,
    on_event: EventSink<TaskItemReport>,
) -> Result<(), String> {
    run_custom_download(manager, url, destination, 0, task_id, on_event).await
}

async fn run_custom_download(
    manager: &DownloadManager,
    url: String,
    destination: PathBuf,
    min_size: u64,
    task_id: i32,
    on_event: EventSink<TaskItemReport>,
) -> Result<(), String> {
    tasks::run(task_id, async {
        let parent = destination
            .parent()
            .filter(|path| path.is_absolute() && path.is_dir())
            .ok_or_else(|| "Invalid destination folder".to_string())?;
        let name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or_else(|| "Invalid filename".to_string())?;
        crate::io::files::validate_file_name(name).map_err(|error| error.to_string())?;
        if destination
            .try_exists()
            .map_err(|error| error.to_string())?
        {
            return Err("Destination already exists".into());
        }
        let url = reqwest::Url::parse(&url).map_err(|error| error.to_string())?;
        manager
            .download_with_progress(
                DownloadConfig {
                    file_index: 0,
                    out_path: destination.clone(),
                    task_item_id: 0,
                    info: DownloadFile {
                        url: url.to_string(),
                        size: 0,
                        sha1: String::new(),
                        path: None,
                    },
                },
                |bytes, total, speed| {
                    on_event(TaskItemReport {
                        task_id,
                        item_id: 0,
                        files_remaining: 1,
                        progress: total
                            .filter(|size| *size > 0)
                            .map_or(0.0, |size| (bytes as f64 / size as f64).min(1.0)),
                        status: TaskStatus::Running,
                        speed: Some(speed),
                    });
                },
            )
            .await
            .map_err(|error| error.to_string())?;
        let size = tokio::fs::metadata(&destination)
            .await
            .map_err(|error| error.to_string())?
            .len();
        if size < min_size {
            let _ = tokio::fs::remove_file(&destination).await;
            return Err("Downloaded file is too small".into());
        }
        let _ = parent;
        Ok(())
    })
    .await
}

pub async fn open_custom_download_folder(directory: PathBuf) -> Result<(), String> {
    if !directory.is_absolute() || !directory.is_dir() {
        return Err("Invalid destination folder".into());
    }
    crate::io::files::open_directory(&directory)
}

async fn report(
    progress: ProgressSink<'_>,
    bytes: u64,
    total_bytes: Option<u64>,
    status: TaskStatus,
) {
    if let Some((options, tx)) = progress {
        let _ = tx
            .send(ProgressUpdate {
                file_index: options.file_index,
                item_id: options.task_item_id,
                progress: FileProgress {
                    downloaded_bytes: bytes,
                    status,
                },
                total_bytes,
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
        if previous.status != TaskStatus::Completed
            && progress.status == TaskStatus::Completed
            && self.remaining > 0
        {
            self.remaining -= 1;
        }
        self.failed |= progress.status == TaskStatus::Failed;
        self.files[index] = FileProgress {
            downloaded_bytes: progress.downloaded_bytes.min(size),
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
