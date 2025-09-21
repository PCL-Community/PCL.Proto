//! The downloader mod.
//! Designed for Minecraft version and mod downloads.
//! Supports mutli task parallel and mspc channel.
use crate::{
    core::api_client::game::{DownloadInfo, VersionDetails},
    setup::ConfigManager,
};
use futures_util::StreamExt;
use reqwest::Client;
use std::{
    collections::HashMap,
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::AsyncWriteExt,
    sync::{Mutex, mpsc},
};

// ---------------- 🌟 Progress Types 🌟 ----------------

#[derive(serde::Serialize)]
pub struct FileProgress {
    pub downloaded_bytes: u64,
    pub status: TaskStatus,
}

impl Default for FileProgress {
    fn default() -> Self {
        Self {
            downloaded_bytes: 0,
            // total_bytes: None,
            status: TaskStatus::Pending,
        }
    }
}

#[derive(PartialEq, Debug, serde_repr::Serialize_repr)]
#[repr(u8)]
pub enum TaskStatus {
    Pending = 0,
    Running = 1,
    Completed = 2,
    Failed = 3,
}

/// inner progress update structure
pub struct ProgressUpdate {
    pub file_index: usize,
    pub progress: FileProgress,
    pub item_id: i32,
}

// ---------------- 🌟 Downloader 🌟 ----------------

/// options to perform a download action
pub struct DownloadConfig {
    pub file_index: usize,
    pub out_path: PathBuf,
    pub task_item_id: i32,
    pub info: DownloadInfo,
}

/// one file to be downloaded in the TaskItem files
#[derive(serde::Serialize)]
pub struct DownloadFile {
    progress: FileProgress,
    info: DownloadInfo,
}
#[derive(Clone)]
pub struct Downloader {
    client: Client,
}

impl Downloader {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// start downloading a single file
    pub async fn start_download(
        &self,
        options: DownloadConfig,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<(), Box<dyn Error>> {
        progress_tx
            .send(ProgressUpdate {
                file_index: options.file_index,
                progress: FileProgress {
                    downloaded_bytes: 0,
                    // total_bytes: None,
                    status: TaskStatus::Pending,
                },
                item_id: options.task_item_id,
            })
            .await?;
        if options.out_path.exists() {
            log::debug!("file exists, skip {:?}", options.out_path);
        } else {
            self.http_download_inner(&options, progress_tx.clone())
                .await?;
        }

        if !crate::util::file::check_sha1(&options.out_path, &options.info.sha1)? {
            log::error!("sha1 check failed! {:?}", options.out_path);
            fs::remove_file(&options.out_path)?;
            progress_tx
                .send(ProgressUpdate {
                    file_index: options.file_index,
                    progress: FileProgress {
                        downloaded_bytes: 0,
                        // total_bytes: None,
                        status: TaskStatus::Failed,
                    },
                    item_id: options.task_item_id,
                })
                .await?;
            let error_notice = format!("sha1 check failed! {:?}", options.out_path);
            log::error!("{error_notice}");
            return Err(error_notice.into());
        }

        progress_tx
            .send(ProgressUpdate {
                file_index: options.file_index,
                progress: FileProgress {
                    downloaded_bytes: 0,
                    // total_bytes: None,
                    status: TaskStatus::Completed,
                },
                item_id: options.task_item_id,
            })
            .await?;
        Ok(())
    }

    /// the actual process of downloading a single file
    async fn http_download_inner(
        &self,
        option: &DownloadConfig,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.client.get(&option.info.url).send().await?;
        let parent_path = option.out_path.parent().unwrap();
        if !parent_path.is_dir() {
            fs::create_dir_all(parent_path)?;
        }
        let mut file = tokio::fs::File::create(&option.out_path).await?;
        // if let Some(total_size) = response.content_length() {
        progress_tx
            .send(ProgressUpdate {
                file_index: option.file_index,
                progress: FileProgress {
                    downloaded_bytes: 0,
                    // total_bytes: Some(total_size),
                    status: TaskStatus::Running,
                },
                item_id: option.task_item_id,
            })
            .await?;
        let mut stream = response.bytes_stream();
        let mut downloaded: u64 = 0;
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            progress_tx
                .send(ProgressUpdate {
                    file_index: option.file_index,
                    progress: FileProgress {
                        downloaded_bytes: downloaded,
                        // total_bytes: Some(total_size),
                        status: TaskStatus::Running,
                    },
                    item_id: option.task_item_id,
                })
                .await?;
        }
        progress_tx
            .send(ProgressUpdate {
                file_index: option.file_index,
                progress: FileProgress {
                    downloaded_bytes: downloaded,
                    // total_bytes: Some(total_size),
                    status: TaskStatus::Completed,
                },
                item_id: option.task_item_id,
            })
            .await?;
        // }
        Ok(())
    }
}

// ---------------- 🌟 TaskItem 🌟 ----------------

#[derive(serde::Serialize)]
pub struct TaskItem {
    pub id: i32,
    pub task_id: i32,
    pub name: String,
    pub progress: f64,
    pub files: Vec<DownloadFile>,
    pub out_dir: PathBuf,
}
impl TaskItem {
    /// create a new TaskItem with several urls to download
    pub fn build_with_infos(
        id: i32,
        task_id: i32,
        name: impl Into<String>,
        download_infos: Vec<DownloadInfo>,
        out_dir: impl Into<PathBuf>,
    ) -> (Arc<Mutex<Self>>, Vec<DownloadConfig>) {
        let files = download_infos
            .into_iter()
            .map(|download_item| DownloadFile {
                progress: FileProgress::default(),
                info: download_item,
            })
            .collect();
        let new_item = Self {
            name: name.into(),
            progress: 0.0,
            files,
            out_dir: out_dir.into(),
            id,
            task_id,
        };
        let download_options = new_item.create_download_options();
        (Mutex::new(new_item).into(), download_options)
    }

    /// update the file progress at specific index
    pub fn update_file_progress(&mut self, index: usize, progress: FileProgress) -> TaskItemReport {
        self.files[index].progress = progress;
        let remaining: usize;
        (self.progress, remaining) = self.calculate_item_progress();
        TaskItemReport {
            task_id: self.task_id,
            item_id: self.id,
            overall_progress: self.progress,
            files_remaining: remaining,
            status: if remaining > 0 {
                TaskStatus::Running
            } else {
                TaskStatus::Completed
            },
        }
    }

    /// TODO)) calculate the progress according to each file's size
    fn calculate_item_progress(&self) -> (f64, usize) {
        let total_files = self.files.len();
        if total_files == 0 {
            return (1.0, 0);
        }

        let mut weighted_progress = 0.0;
        let mut remaining = total_files;

        for file in &self.files {
            match &file.progress.status {
                TaskStatus::Completed => {
                    weighted_progress += 1.0;
                    remaining -= 1;
                }
                TaskStatus::Running => {
                    let file_progress = if file.info.size > 0 {
                        file.progress.downloaded_bytes as f64 / file.info.size as f64
                    } else {
                        1.0
                    };
                    weighted_progress += file_progress;
                }
                _ => {}
            }
        }

        (weighted_progress / total_files as f64, remaining)
    }

    /// generate download options
    fn create_download_options(&self) -> Vec<DownloadConfig> {
        self.files
            .iter()
            .enumerate()
            .map(|(index, file)| DownloadConfig {
                task_item_id: self.id,
                // url: file.info.url.clone(),
                file_index: index,
                out_path: self.out_dir.join(Path::new(
                    &file
                        .info
                        .path
                        .as_deref()
                        .unwrap_or(&file.info.url.split('/').last().unwrap_or(&file.info.url)),
                )),
                info: file.info.clone(),
            })
            .collect()
    }
}

// ---------------- 🌟 ProgressMonitor 🌟 ----------------

#[derive(Default)]
pub struct ProgressMonitor {
    task_items: HashMap<i32, Arc<Mutex<TaskItem>>>,
}

impl ProgressMonitor {
    pub async fn with_item(mut self, task: Arc<Mutex<TaskItem>>) -> Self {
        let id = {
            let task_guard = task.lock().await;
            task_guard.id
        };
        self.task_items.insert(id, task);
        self
    }

    pub async fn start_monitoring(
        &self,
        mut progress_rx: mpsc::Receiver<ProgressUpdate>,
        on_event: tauri::ipc::Channel<TaskItemReport>,
    ) {
        // restrain the frequency to 10 send per secend
        const MIN_INTERVAL_MS: u128 = 100;
        let mut last_sent_time = tokio::time::Instant::now();
        let mut pending_report: Option<TaskItemReport>;
        while let Some(update) = progress_rx.recv().await {
            let mut task_guard = self
                .task_items
                .get(&update.item_id)
                .expect("[progress minitor] wrong item id's got!!!")
                .lock()
                .await;
            let report = task_guard.update_file_progress(update.file_index, update.progress);
            pending_report = Some(report);
            if let Some(report) = pending_report.take() {
                let now = tokio::time::Instant::now();
                if report.status == TaskStatus::Completed
                    || report.status == TaskStatus::Failed
                    || now.duration_since(last_sent_time).as_millis() >= MIN_INTERVAL_MS
                {
                    on_event.send(report).expect("report corrupted");
                    last_sent_time = now;
                }
            }
        }
    }
}

// ---------------- 🌟 The Minecraft Download Event 🌟 ----------------

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskItemReport {
    pub task_id: i32,
    pub item_id: i32,
    pub files_remaining: usize,
    pub overall_progress: f64,
    pub status: TaskStatus,
}

fn try_get_temp_json(
    version_id: &str,
    version_folder: &Path,
) -> Result<VersionDetails, Box<dyn Error>> {
    let temp_json = std::env::temp_dir().join(format!("pcl-proto-{0}/{0}.json", &version_id));
    let reader = fs::File::open(&temp_json)?;
    let details: VersionDetails = serde_json::from_reader(reader)?;
    fs::copy(
        temp_json,
        version_folder.join(format!("{}.json", version_id)),
    )?;
    Ok(details)
}

/// This command would receive a version_id of the game to download.
/// The frontend would provide a task_id and a channel for the progress feedback, and the progress happended should be sent throught the channel.
/// ## The four task items are fixed that:
/// 1. Fetch the json indicating the version details
/// 2. Download the version jar file
/// 3. Download the libraries to support the version
/// 4. Download the resources
#[tauri::command(rename_all = "snake_case")]
pub async fn download_minecraft_version(
    state: tauri::State<'_, Arc<std::sync::Mutex<crate::setup::AppState>>>,
    on_event: tauri::ipc::Channel<TaskItemReport>,
    version_id: &str,
    task_id: i32,
) -> Result<(), String> {
    log::info!("start a task of downloading mc: {}", version_id);
    // get the folder of this version
    let repo = state
        .lock()
        .map_err(|err| err.to_string())?
        .active_repo_path
        .clone();
    let version_folder = repo.join(format!("versions/{}", version_id));

    // compose the task items
    // get the download info
    let (jar_download, libraries_download) = {
        let version_details: VersionDetails;
        if let Ok(version_details_tmp) = try_get_temp_json(&version_id, &version_folder) {
            version_details = version_details_tmp;
        } else {
            version_details = ConfigManager::instance()
                .api_client
                .get_version_details(&version_id, &version_folder)
                .await
                .map_err(|err| err.to_string())?;
        }
        let libraries = version_details.libraries;
        // modify the jar path cause api don't provide it
        let mut jar_download = version_details.downloads.client;
        jar_download.path = Some(format!("{}.jar", version_id));
        (
            jar_download,
            libraries
                .iter()
                .map(|lib| lib.downloads.artifact.to_owned())
                .collect::<Vec<DownloadInfo>>(),
        )
    };
    // report the first task item: get the version json
    on_event
        .send(TaskItemReport {
            task_id,
            item_id: 0,
            files_remaining: 0,
            overall_progress: 1.0,
            status: TaskStatus::Completed,
        })
        .map_err(|err| err.to_string())?;

    // the task info is fixed
    let (task1, download_options1) =
        TaskItem::build_with_infos(1, task_id, "jar", vec![jar_download], version_folder);
    let (task2, download_options2) = TaskItem::build_with_infos(
        2,
        task_id,
        "libraries",
        libraries_download,
        repo.join("libraries"),
    );

    // set up the minotor
    let (progress_tx, progress_rx) = mpsc::channel(100);
    let monitor = ProgressMonitor::default()
        .with_item(Arc::clone(&task1))
        .await
        .with_item(Arc::clone(&task2))
        .await;
    let monitor_handle = tokio::task::spawn(async move {
        monitor.start_monitoring(progress_rx, on_event).await;
    });

    // start the actual downloading
    let downloader = Downloader::new();
    let mut download_handles = Vec::new();
    for options_all in [download_options1, download_options2] {
        for options in options_all {
            let tx = progress_tx.clone(); // light weight clone
            let downloader = downloader.clone(); // light weight clone
            let handle = tokio::task::spawn(async move {
                if let Err(e) = downloader.start_download(options, tx).await {
                    log::error!("{}", e);
                }
            });
            download_handles.push(handle);
        }
    }

    // wait until all the procedures have been finished
    for handle in download_handles {
        handle.await.map_err(|err| err.to_string())?;
    }
    drop(progress_tx);
    monitor_handle.await.map_err(|err| err.to_string())?;
    let final_task1 = task1.lock().await;
    let final_task2 = task2.lock().await;
    log::info!(
        "successfully downloaded {} files!",
        final_task2.files.len() + final_task1.files.len()
    );
    Ok(())
}
