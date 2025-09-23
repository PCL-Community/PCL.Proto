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
        if options.out_path.exists()
            && crate::util::file::check_sha1(&options.out_path, &options.info.sha1)
                .is_ok_and(|result| result == true)
        {
            log::debug!("file exists, skip {:?}", options.out_path);
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
            return Ok(());
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

    /// simply download a file and check its sha1
    async fn download_without_report(
        &self,
        info: &DownloadInfo,
        base_path: &Path,
    ) -> Result<PathBuf, Box<dyn Error>> {
        let response = self.client.get(&info.url).send().await?;
        let bytes = response.bytes().await?;
        let out_path = base_path.join(
            info.path
                .as_deref()
                .unwrap_or(info.url.split('/').last().unwrap_or(&info.url).into()),
        );
        // return directly if there is already the right file
        if let Ok(sha1_check) = crate::util::file::check_sha1(&out_path, &info.sha1)
            && sha1_check
        {
            return Ok(out_path);
        }
        let parent_path = out_path.parent().unwrap();
        if !parent_path.is_dir() {
            fs::create_dir_all(parent_path)?;
        }
        let mut file = tokio::fs::File::create(&out_path).await?;
        file.write(&bytes).await?;
        file.flush().await?;
        drop(file);
        if !crate::util::file::check_sha1(&out_path, &info.sha1)? {
            Err("sha1 mismatch".into())
        } else {
            Ok(out_path)
        }
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
    pub total_size: u64, // for convenience of progress calculate
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
        let total_size = download_infos.iter().fold(0, |acc, info| acc + info.size);
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
            total_size,
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

    /// calculate the progress according to each file's size
    fn calculate_item_progress(&self) -> (f64, usize) {
        let downloaded = self.files.iter().fold(0, |acc, file| {
            acc + match file.progress.status {
                TaskStatus::Completed => file.info.size,
                TaskStatus::Running => file.progress.downloaded_bytes,
                _ => 0,
            }
        });
        let remaining = self
            .files
            .iter()
            .filter(|file| file.progress.status != TaskStatus::Completed)
            .count();
        (downloaded as f64 / self.total_size as f64, remaining)
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

pub mod minecraft_resource {
    use super::*;

    /// get cached version json from the temp dir if it exists,
    /// then copy it to the vesion folder and return the value of it
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

    ///
    // fn fetch_assets_index()

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
        let downloader = Downloader::new();
        log::info!("start a task of downloading mc: {}", version_id);
        // get the folder of this version
        let repo = state
            .lock()
            .map_err(|err| err.to_string())?
            .active_repo_path
            .clone();
        let version_folder = repo.join(format!("versions/{}", version_id));
        let assets_folder = repo.join("assets");

        // compose the task items
        // get the download info
        let (jar_download, libraries_download, assets_download) = {
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

            // fetch the assets index
            // let assets_index_downloadInfo = version_details.asset_index;
            let assets_download = {
                let asset_index_file = downloader
                    .download_without_report(
                        &version_details.asset_index,
                        &assets_folder.join("indexes"),
                    )
                    .await
                    .map_err(|err| err.to_string())?;
                let reader = fs::File::open(asset_index_file).unwrap();
                let asset_index: serde_json::Value = serde_json::from_reader(reader).unwrap();
                let objects = asset_index["objects"].as_object().unwrap();
                let resources_base = ConfigManager::instance()
                    .api_client
                    .api_bases()
                    .resources_base;
                objects
                    .iter()
                    .map(|(_path, value)| {
                        let hash = value["hash"].as_str().unwrap();
                        let size = value["size"].as_u64().unwrap();
                        let path = format!("{}/{}", &hash[..2], hash);
                        DownloadInfo {
                            sha1: hash.to_string(),
                            size,
                            url: format!("{}/{}", resources_base, path),
                            path: Some(path),
                        }
                    })
                    .collect::<Vec<_>>()
            };
            // TODO)) 仅筛选当前平台的库，去除无关平台
            (
                jar_download,
                libraries
                    .iter()
                    .map(|lib| lib.downloads.artifact.to_owned())
                    .collect::<Vec<DownloadInfo>>(),
                assets_download,
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
        let (task3, download_options3) = TaskItem::build_with_infos(
            3,
            task_id,
            "assets",
            assets_download,
            repo.join("assets/objects"),
        );

        // set up the minotor
        let (progress_tx, progress_rx) = mpsc::channel(100);
        let monitor = ProgressMonitor::default()
            .with_item(Arc::clone(&task1))
            .await
            .with_item(Arc::clone(&task2))
            .await
            .with_item(Arc::clone(&task3))
            .await;
        let monitor_handle = tokio::task::spawn(async move {
            monitor.start_monitoring(progress_rx, on_event).await;
        });

        // start the actual downloading
        let max_concurrent_downloads = 15;
        let mut all_options = Vec::new();
        all_options.extend(download_options1);
        all_options.extend(download_options2);
        all_options.extend(download_options3);
        let download_stream = futures_util::stream::iter(all_options.into_iter().map(|options| {
            let downloader = downloader.clone();
            let tx = progress_tx.clone();
            async move {
                if let Err(e) = downloader.start_download(options, tx).await {
                    log::error!("{}", e);
                }
            }
        }));
        // wait until all the tasks has finished
        let _ = download_stream
            .buffer_unordered(max_concurrent_downloads)
            .collect::<Vec<_>>()
            .await;
        drop(progress_tx);
        monitor_handle.await.map_err(|err| err.to_string())?;
        log::info!(
            "successfully downloaded {} files!",
            task1.lock().await.files.len()
                + task2.lock().await.files.len()
                + task3.lock().await.files.len()
        );
        Ok(())
    }
}
