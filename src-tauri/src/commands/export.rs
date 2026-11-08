//! 整合包 / 客户端 / 服务端导出类命令。

use std::path::PathBuf;

use pcl_core::io::download::TaskItemReport;
use pcl_core::io::net::http::api_client::MinecraftApiClient;
use pcl_core::minecraft::export::{loader, resource, version};
use pcl_core::minecraft::version::DownloadFile;

#[tauri::command(rename_all = "snake_case")]
pub async fn download_resource(
    client: tauri::State<'_, &MinecraftApiClient>,
    info: DownloadFile,
    destination: PathBuf,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<(), String> {
    resource::download_resource(
        &client.downloader().await,
        info,
        destination,
        task_id,
        super::channel_sink(on_event),
    )
    .await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn export_minecraft_version(
    client: tauri::State<'_, &MinecraftApiClient>,
    version_id: String,
    directory: PathBuf,
    server: bool,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<String, String> {
    version::export_minecraft_version(
        &client,
        version_id,
        directory,
        server,
        task_id,
        super::channel_sink(on_event),
    )
    .await
}

#[tauri::command]
pub async fn open_minecraft_wiki(url: String) -> Result<(), String> {
    let parsed = reqwest::Url::parse(&url).map_err(|error| error.to_string())?;
    if parsed.scheme() != "https"
        || !parsed
            .host_str()
            .is_some_and(|host| host == "minecraft.wiki" || host.ends_with(".minecraft.wiki"))
        || !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.port().is_some()
    {
        return Err("Invalid Minecraft Wiki URL".into());
    }
    crate::external::open_async(parsed.to_string()).await
}

#[tauri::command]
pub async fn save_loader_file(
    loader: String,
    game: String,
    version: String,
    destination: PathBuf,
) -> Result<(), String> {
    loader::save_loader_file(
        &crate::config::manager::ConfigManager::instance()
            .api_client
            .downloader()
            .await,
        loader,
        game,
        version,
        destination,
    )
    .await
}

#[tauri::command]
pub async fn open_loader_changelog(
    loader: String,
    game: String,
    version: String,
) -> Result<(), String> {
    let url = loader::changelog_url(&loader, &game, &version)?;
    crate::external::open_async(url).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ipc_accepts_task_manager_argument_names() {
        let client: &'static MinecraftApiClient = Box::leak(Box::new(MinecraftApiClient::new(
            pcl_core::io::net::http::HttpTransport::default(),
            &pcl_core::io::net::http::api_client::ApiProvider::Official,
        )));
        let app = tauri::test::mock_builder()
            .manage(client)
            .invoke_handler(tauri::generate_handler![download_resource])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let result = tauri::test::get_ipc_response(
            &window,
            tauri::webview::InvokeRequest {
                cmd: "download_resource".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: window.url().unwrap(),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({
                    "info": { "url": "https://example.org/file.jar", "size": 1, "sha1": "", "path": null },
                    "destination": "/tmp/resource-test.jar",
                    "task_id": 1,
                    "on_event": "__CHANNEL__:1"
                })),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );
        assert_eq!(result.unwrap_err(), "Unsupported resource host");
    }
}
