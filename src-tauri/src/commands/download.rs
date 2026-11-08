//! 下载任务与下载设置类命令。
use crate::config::download::DownloadSettings;

use std::path::PathBuf;

use crate::config::manager::ConfigManager;

use pcl_core::io::{
    download::{self, TaskItemReport, tasks},
    net::http::resources::DownloadStats,
};
use pcl_core::minecraft::mod_loader::FabricFamily;

#[tauri::command(rename_all = "snake_case")]
pub async fn download_fabric_installer(
    family: FabricFamily,
    version: String,
    destination: PathBuf,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<(), String> {
    download::download_fabric_installer(
        &ConfigManager::instance().api_client.downloader(),
        family,
        version,
        destination,
        task_id,
        super::channel_sink(on_event),
    )
    .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_custom_file(
    url: String,
    destination: PathBuf,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<(), String> {
    let url = crate::external::validate(&url)?;
    download::download_custom_file(
        &ConfigManager::instance().api_client.downloader(),
        url,
        destination,
        task_id,
        super::channel_sink(on_event),
    )
    .await
}

#[tauri::command]
pub async fn open_custom_download_folder(directory: PathBuf) -> Result<(), String> {
    download::open_custom_download_folder(directory).await
}

#[tauri::command(rename_all = "snake_case")]
pub fn prepare_download_task(task_id: i32) -> Result<(), String> {
    tasks::prepare_download_task(task_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn cancel_download_task(task_id: i32) -> Result<(), String> {
    tasks::cancel_download_task(task_id)
}

#[tauri::command(rename_all = "snake_case")]
pub fn release_download_task(task_id: i32) {
    tasks::release_download_task(task_id)
}

#[tauri::command]
pub fn get_download_stats() -> DownloadStats {
    ConfigManager::instance().api_client.resources().stats()
}

#[tauri::command]
pub async fn set_download_settings(settings: DownloadSettings) -> Result<DownloadSettings, String> {
    tokio::task::spawn_blocking(move || {
        ConfigManager::instance().update_download_settings(settings)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_download_settings() -> Option<DownloadSettings> {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .download_settings
        .clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn custom_download_ipc_accepts_task_argument_names() {
        let app = tauri::test::mock_builder()
            .invoke_handler(tauri::generate_handler![download_custom_file])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "custom", Default::default())
            .build()
            .unwrap();
        let result = tauri::test::get_ipc_response(
            &window,
            tauri::webview::InvokeRequest {
                cmd: "download_custom_file".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: window.url().unwrap(),
                body: tauri::ipc::InvokeBody::Json(
                    serde_json::json!({"url":"file:///bad", "destination":"/tmp/test", "task_id":1, "on_event":"__CHANNEL__:1"}),
                ),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );
        assert_eq!(result.unwrap_err(), "Invalid external URL");
    }
}
