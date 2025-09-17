use super::api_client::McApiError;
use futures_util::stream::StreamExt;
use reqwest::Client;
use std::{
    path::PathBuf,
    sync::{Arc, LazyLock},
};
use tokio::io::AsyncWriteExt;

#[derive(Default)]
pub struct DownloadOptions {
    pub output_dir: PathBuf,
    pub file_name: Option<String>,
    pub overwrite: bool,
    pub progress_callback: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
}

impl DownloadOptions {
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            ..Default::default()
        }
    }

    pub fn with_progress_callback(
        mut self,
        callback: impl Fn(u64, u64) + Send + Sync + 'static,
    ) -> Self {
        self.progress_callback = Some(Box::new(callback));
        self
    }

    pub fn set_file_name(&mut self, file_name: impl Into<String>) -> &mut Self {
        self.file_name = Some(file_name.into());
        self
    }

    pub fn overwrite(mut self, overwrite: bool) -> Self {
        self.overwrite = overwrite;
        self
    }
}

pub struct Downloader {
    pub client: Arc<Client>,
}

pub static DOWNLOADER: LazyLock<Downloader> = LazyLock::new(|| Downloader::new());

impl Downloader {
    fn new() -> Self {
        Self {
            client: Arc::new(Client::new()),
        }
    }

    pub async fn download_file(
        &self,
        url: &str,
        options: &DownloadOptions,
    ) -> Result<PathBuf, McApiError> {
        tokio::fs::create_dir_all(&options.output_dir).await?;
        let file_name = options
            .file_name
            .as_deref()
            .unwrap_or(url.split('/').last().unwrap_or("download"));
        let output_path = options.output_dir.join(file_name);
        if output_path.exists() && !options.overwrite {
            return Ok(output_path);
        }
        let response = self.client.get(url).send().await?;
        if !response.status().is_success() {
            return Err(McApiError::DownloadFailed(format!(
                "HTTP status: {}",
                response.status()
            )));
        }
        let total_size = response.content_length().ok_or(McApiError::DownloadFailed(
            "Unknown content length".to_string(),
        ))?;
        let mut file = tokio::fs::File::create(&output_path).await?;
        let mut downloaded = 0 as u64;
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            downloaded += chunk.len() as u64;
            if let Some(callback) = &options.progress_callback {
                callback(downloaded, total_size);
            }
        }
        Ok(output_path)
    }
}

#[tokio::test]
async fn jar_download_test() {
    let url = "https://piston-data.mojang.com/v1/objects/a19d9badbea944a4369fd0059e53bf7286597576/client.jar";
    let output_dir = std::env::temp_dir();
    let options = DownloadOptions {
        output_dir,
        file_name: Some("client.jar".to_string()),
        overwrite: true,
        progress_callback: Some(Box::new(|downloaded, total| {
            let percent = (downloaded as f64 / total as f64) * 100.0;
            println!("Downloaded: {:.2}%", percent);
        })),
    };
    let downloader = &DOWNLOADER;
    let output_path = downloader.download_file(url, &options).await.unwrap();
    assert!(output_path.exists());
}
