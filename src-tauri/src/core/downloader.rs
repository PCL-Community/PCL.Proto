use futures_util::StreamExt;
use reqwest::Client;
use std::{
    error::Error,
    path::{Path, PathBuf},
    sync::Arc,
};
use tokio::{
    io::AsyncWriteExt,
    sync::{Mutex, mpsc},
};

use crate::core::api_client::game::DownloadInfo;

pub struct FileProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
    pub status: DownloadStatus,
}

impl Default for FileProgress {
    fn default() -> Self {
        Self {
            downloaded_bytes: 0,
            total_bytes: None,
            status: DownloadStatus::Pending,
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum DownloadStatus {
    Pending,
    Downloading,
    Completed,
    Failed(String),
}

pub struct DownloadOptions {
    pub url: String,
    pub file_index: usize,
    pub out_path: PathBuf,
}

pub struct TaskItem {
    pub id: String,
    pub name: String,
    pub overall_progress: f64,
    pub files: Vec<DownloadFile>,
    pub out_dir: PathBuf,
}

pub struct DownloadFile {
    // url: String,
    progress: FileProgress,
    // path: String,
    info: DownloadInfo,
}

pub struct ProgressUpdate {
    pub file_index: usize,
    pub progress: FileProgress,
}

impl TaskItem {
    /// create a new TaskItem with several urls to download
    pub fn build(
        id: impl Into<String>,
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
            id: id.into(),
            name: name.into(),
            overall_progress: 0.0,
            files,
            out_dir: out_dir.into(),
        };
        let download_options = new_item.create_download_options();
        (Mutex::new(new_item).into(), download_options)
    }

    /// update the file progress at specific index
    pub fn update_file_progress(&mut self, index: usize, progress: FileProgress) {
        self.files[index].progress = progress;
        self.calculate_overall_progress();
    }

    fn calculate_overall_progress(&mut self) {
        let total_files = self.files.len();
        if total_files == 0 {
            self.overall_progress = 0.0;
            return;
        }

        let mut weighted_progress = 0.0;

        for file in &self.files {
            match &file.progress.status {
                DownloadStatus::Completed => {
                    weighted_progress += 1.0;
                }
                DownloadStatus::Downloading => {
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

        self.overall_progress = weighted_progress / total_files as f64 * 100.0;
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
                    status: DownloadStatus::Downloading,
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
                        status: DownloadStatus::Completed,
                    },
                })
                .await?;
            println!("file exists! {:?}", option.out_path);
            return Ok(());
        }
        let mut file = tokio::fs::File::create(&option.out_path).await?;
        if let Some(total_size) = response.content_length() {
            progress_tx
                .send(ProgressUpdate {
                    file_index: option.file_index,
                    progress: FileProgress {
                        downloaded_bytes: 0,
                        total_bytes: Some(total_size),
                        status: DownloadStatus::Downloading,
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
                            status: DownloadStatus::Downloading,
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
                        status: DownloadStatus::Completed,
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

    pub async fn start_monitoring(&self, mut progress_rx: mpsc::Receiver<ProgressUpdate>) {
        while let Some(update) = progress_rx.recv().await {
            let mut task_guard = self.task.lock().await;
            task_guard.update_file_progress(update.file_index, update.progress);
            println!(
                "任务 {} - 文件 {} - 总进度: {:.1}%",
                task_guard.name, update.file_index, task_guard.overall_progress
            );
        }
    }
}

#[cfg(test)]
#[tokio::test]
async fn download_jars() -> Result<(), Box<dyn Error>> {
    println!("启动异步下载测试...");
    let downloads = vec![
        DownloadInfo {
            url: "https://piston-data.mojang.com/v1/objects/a19d9badbea944a4369fd0059e53bf7286597576/client.jar".to_string(),
            path: None,
            sha1: "".to_string(),
            size: 0,
        },
        DownloadInfo {
            url: "https://libraries.minecraft.net/ca/weblite/java-objc-bridge/1.1/java-objc-bridge-1.1.jar".to_string(),
            path: None,
            sha1: "".to_string(),
            size: 0,
        },
    ];
    let (task, download_options) =
        TaskItem::build("1", "多文件下载", downloads, "/Users/amagicpear/Downloads");
    let (progress_tx, progress_rx) = mpsc::channel(100);
    let monitor = ProgressMonitor::new(Arc::clone(&task));
    let monitor_handle = tokio::task::spawn(async move {
        monitor.start_monitoring(progress_rx).await;
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
        handle.await?;
    }
    drop(progress_tx);
    monitor_handle.await?;
    let final_task = task.lock().await;
    println!("\n下载完成！");
    for (i, file) in final_task.files.iter().enumerate() {
        println!("文件{}:{:?}", i, file.progress.status);
    }
    Ok(())
}
