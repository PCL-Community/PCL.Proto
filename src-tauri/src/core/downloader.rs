use futures_util::StreamExt;
use reqwest::Client;
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::AsyncWriteExt,
    sync::{Mutex, mpsc},
};

use crate::{
    core::api_client::game::{DownloadInfo, VersionDetails},
    setup::ConfigManager,
};

#[derive(serde::Serialize)]
pub struct FileProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub status: TaskStatus,
}

impl Default for FileProgress {
    fn default() -> Self {
        Self {
            downloaded_bytes: 0,
            total_bytes: None,
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

/// options to perform a download action
pub struct DownloadOptions {
    pub url: String,
    pub file_index: usize,
    pub out_path: PathBuf,
}

#[derive(serde::Serialize)]
pub struct TaskItem {
    pub id: i32,
    pub task_id: i32,
    pub name: String,
    pub progress: f64,
    pub files: Vec<DownloadFile>,
    pub out_dir: PathBuf,
}

/// one file to be downloaded in the TaskItem files
#[derive(serde::Serialize)]
pub struct DownloadFile {
    progress: FileProgress,
    info: DownloadInfo,
}

/// inner progress update structure
pub struct ProgressUpdate {
    pub file_index: usize,
    pub progress: FileProgress,
}

impl TaskItem {
    /// create a new TaskItem with several urls to download
    pub fn build_with_infos(
        id: i32,
        task_id: i32,
        name: impl Into<String>,
        download_infos: Vec<DownloadInfo>,
        out_dir: impl Into<PathBuf>,
    ) -> (Arc<Mutex<Self>>, Vec<DownloadOptions>) {
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
    pub fn update_file_progress(&mut self, index: usize, progress: FileProgress) -> TaskEvent {
        self.files[index].progress = progress;
        let remaining: usize;
        (self.progress, remaining) = self.calculate_overall_progress();
        TaskEvent {
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

    fn calculate_overall_progress(&self) -> (f64, usize) {
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
                    let file_progress = if let Some(total) = file.progress.total_bytes {
                        if total > 0 {
                            file.progress.downloaded_bytes as f64 / total as f64
                        } else {
                            0.0
                        }
                    } else {
                        0.5
                    };
                    weighted_progress += file_progress;
                }
                _ => {}
            }
        }

        (weighted_progress / total_files as f64, remaining)
    }

    /// generate download options
    fn create_download_options(&self) -> Vec<DownloadOptions> {
        self.files
            .iter()
            .enumerate()
            .map(|(index, file)| DownloadOptions {
                url: file.info.url.clone(),
                file_index: index,
                out_path: self.out_dir.join(Path::new(
                    &file
                        .info
                        .path
                        .as_deref()
                        .unwrap_or(&file.info.url.split('/').last().unwrap_or(&file.info.url)),
                )),
            })
            .collect()
    }
}
#[derive(Clone)]
pub struct DownloadManager {
    client: Client,
}

impl DownloadManager {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    /// start downloading a single file
    pub async fn start_download(
        &self,
        options: DownloadOptions,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<(), Box<dyn Error>> {
        progress_tx
            .send(ProgressUpdate {
                file_index: options.file_index,
                progress: FileProgress {
                    downloaded_bytes: 0,
                    total_bytes: None,
                    status: TaskStatus::Running,
                },
            })
            .await?;
        self.http_download_inner(&options, progress_tx).await?;
        Ok(())
    }

    async fn http_download_inner(
        &self,
        option: &DownloadOptions,
        progress_tx: mpsc::Sender<ProgressUpdate>,
    ) -> Result<(), Box<dyn Error>> {
        let response = self.client.get(&option.url).send().await?;
        if option.out_path.exists() {
            progress_tx
                .send(ProgressUpdate {
                    file_index: option.file_index,
                    progress: FileProgress {
                        downloaded_bytes: 0,
                        total_bytes: None,
                        status: TaskStatus::Completed,
                    },
                })
                .await?;
            log::warn!("file exists! {:?}", option.out_path);
            return Ok(());
        }
        let parent_path = option.out_path.parent().unwrap();
        if !option.out_path.parent().unwrap().is_dir() {
            fs::create_dir_all(parent_path)?;
        }
        let mut file = tokio::fs::File::create(&option.out_path).await?;
        if let Some(total_size) = response.content_length() {
            progress_tx
                .send(ProgressUpdate {
                    file_index: option.file_index,
                    progress: FileProgress {
                        downloaded_bytes: 0,
                        total_bytes: Some(total_size),
                        status: TaskStatus::Running,
                    },
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
                            total_bytes: Some(total_size),
                            status: TaskStatus::Running,
                        },
                    })
                    .await?;
            }
            progress_tx
                .send(ProgressUpdate {
                    file_index: option.file_index,
                    progress: FileProgress {
                        downloaded_bytes: downloaded,
                        total_bytes: Some(total_size),
                        status: TaskStatus::Completed,
                    },
                })
                .await?;
        }
        Ok(())
    }
}

pub struct ProgressMonitor {
    task: Arc<Mutex<TaskItem>>,
}

impl ProgressMonitor {
    pub fn new(task: Arc<Mutex<TaskItem>>) -> Self {
        Self { task }
    }

    pub async fn start_monitoring(
        &self,
        mut progress_rx: mpsc::Receiver<ProgressUpdate>,
        on_event: tauri::ipc::Channel<TaskEvent>,
    ) {
        while let Some(update) = progress_rx.recv().await {
            let mut task_guard = self.task.lock().await;
            let report = task_guard.update_file_progress(update.file_index, update.progress);
            on_event.send(report).expect("report corrupted");
        }
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub struct TaskEvent {
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

#[tauri::command(rename_all = "snake_case")]
pub async fn download_minecraft_version(
    state: tauri::State<'_, Arc<std::sync::Mutex<crate::setup::AppState>>>,
    on_event: tauri::ipc::Channel<TaskEvent>,
    version_id: &str,
) -> Result<(), String> {
    log::info!("start a task of downloading mc: {}", version_id);
    // get the folder of this version
    let (repo, version_folder) = {
        let guard = state.lock().map_err(|err| err.to_string())?;
        (
            guard.active_repo_path.clone(),
            guard.active_repo_path.join(version_id),
        )
    };
    if !version_folder.exists() {
        fs::create_dir(&version_folder).map_err(|err| err.to_string())?;
    } else {
        log::warn!("downloading mc in an exsiting directory!")
    }
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
        (
            version_details.downloads.client,
            libraries
                .iter()
                .map(|lib| lib.downloads.artifact.to_owned())
                .collect::<Vec<DownloadInfo>>(),
        )
    };
    let (task, download_options) =
        TaskItem::build_with_infos(0, 0, "多文件下载", libraries_download, version_folder);
    // {
    //     let task_ref = &task.lock().await;
    //     on_event
    //         .send(TaskEvent {
    //             task_id: 0,
    //             item_id: 0,
    //             files_remaining: 0,
    //             overall_progress: 0.0,
    //             status: TaskStatus::Failed,
    //         })
    //         .unwrap();
    // }
    let (progress_tx, progress_rx) = mpsc::channel(100);
    let monitor = ProgressMonitor::new(Arc::clone(&task));
    let monitor_handle = tokio::task::spawn(async move {
        monitor.start_monitoring(progress_rx, on_event).await;
    });
    let download_manager = DownloadManager::new();
    let mut download_handles = Vec::new();
    for options in download_options {
        let tx = progress_tx.clone();
        let manager = download_manager.clone();
        let handle = tokio::task::spawn(async move {
            if let Err(e) = manager.start_download(options, tx).await {
                eprintln!("下载错误: {}", e);
            }
        });
        download_handles.push(handle);
    }
    for handle in download_handles {
        handle.await.map_err(|err| err.to_string())?;
    }
    drop(progress_tx);
    monitor_handle.await.map_err(|err| err.to_string())?;
    let final_task = task.lock().await;
    println!("\n下载完成！");
    for (i, file) in final_task.files.iter().enumerate() {
        println!("文件{}:{:?}", i, file.progress.status);
    }
    Ok(())
}
