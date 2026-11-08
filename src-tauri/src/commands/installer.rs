//! 安装流程类命令：原版安装、Forge 系安装器、整合包导入。

use std::path::PathBuf;
use std::sync::Arc;

use crate::config::manager::ConfigManager;
use crate::config::states::AppState;
use pcl_core::io::download::TaskItemReport;
use pcl_core::io::net::http::api_client::MinecraftApiClient;
use pcl_core::minecraft::installer::{
    self, LoaderInstall,
    forgelike::{self, JavaInstallOffer},
    modpack::{self, OnlinePackInfo, PackInfo},
};
use pcl_core::minecraft::version::DownloadFile;
use tokio::sync::Mutex as AsyncMutex;

#[derive(serde::Serialize)]
pub struct InstallResult {
    selected: bool,
    selection_error: Option<String>,
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)] // IPC 参数固定
pub async fn download_minecraft_version(
    state: tauri::State<'_, Arc<AsyncMutex<AppState>>>,
    client: tauri::State<'_, &MinecraftApiClient>,
    on_event: tauri::ipc::Channel<TaskItemReport>,
    version_id: String,
    task_id: i32,
    instance_name: String,
    loader: Option<LoaderInstall>,
) -> Result<InstallResult, String> {
    let repo = state.lock().await.active_repo_path.clone();
    let java = crate::config::java::installer_candidates(&state).await;
    let paths = ConfigManager::instance().java_runtime_dir();
    let bridge =
        crate::assets::Assets::new(ConfigManager::instance().config_dir()).forge_installer();
    let outcome = installer::download_minecraft_version(
        &repo,
        &client,
        super::channel_sink(on_event),
        version_id,
        task_id,
        instance_name.clone(),
        loader,
        &java,
        &paths,
        bridge.as_deref(),
    )
    .await?;
    if outcome.isolated {
        let directory = outcome.directory.clone();
        tokio::task::spawn_blocking(move || {
            ConfigManager::instance().update(move |state| {
                state.instance_isolation.insert(directory, true);
                Ok(())
            })
        })
        .await
        .map_err(|error| error.to_string())??;
    }
    // 安装过程可能自动装了 Java 运行时，刷新一次写回配置。
    if let Err(error) = crate::config::java::refresh(true).await {
        log::warn!("Cannot refresh Java after install: {error}");
    }
    let should_select = ConfigManager::instance()
        .app_state
        .lock()
        .await
        .download_settings
        .as_ref()
        .and_then(|settings| settings.auto_select_instance)
        .unwrap_or(false);
    if !should_select {
        return Ok(InstallResult {
            selected: false,
            selection_error: None,
        });
    }
    let selection = tokio::task::spawn_blocking(move || {
        let repository = pcl_core::minecraft::instance::folder::McFolder::new("", repo.clone());
        let instance = pcl_core::minecraft::instance::McInstance::from_version_folder(
            &repo.join("versions").join(instance_name),
            &repository,
        )
        .map_err(|error| format!("{error:?}"))?;
        ConfigManager::instance().select_installed_instance(&repo, instance)
    })
    .await
    .map_err(|error| error.to_string())
    .and_then(|result| result);
    Ok(match selection {
        Ok(selected) => InstallResult {
            selected,
            selection_error: None,
        },
        Err(error) => InstallResult {
            selected: false,
            selection_error: Some(error),
        },
    })
}

#[tauri::command(rename_all = "snake_case")]
pub async fn forgelike_installer_name(
    client: tauri::State<'_, &MinecraftApiClient>,
    game: String,
    loader: LoaderInstall,
) -> Result<String, String> {
    forgelike::forgelike_installer_name(&client, game, loader).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn download_forgelike_installer(
    client: tauri::State<'_, &MinecraftApiClient>,
    game: String,
    loader: LoaderInstall,
    destination: PathBuf,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<(), String> {
    forgelike::download_forgelike_installer(
        &client,
        game,
        loader,
        destination,
        task_id,
        super::channel_sink(on_event),
    )
    .await?;
    // 安装器可能顺带下载了 Java 运行时，刷新一次写回配置。
    if let Err(error) = crate::config::java::refresh(true).await {
        log::warn!("Cannot refresh Java after installer download: {error}");
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn prepare_forgelike_java(
    state: tauri::State<'_, Arc<AsyncMutex<AppState>>>,
    client: tauri::State<'_, &MinecraftApiClient>,
    version_id: String,
    loader: LoaderInstall,
) -> Result<Option<JavaInstallOffer>, String> {
    forgelike::prepare_forgelike_java(
        &client,
        version_id,
        loader,
        &crate::config::java::installer_candidates(&state).await,
    )
    .await
}

#[tauri::command]
pub async fn inspect_modpack(path: PathBuf) -> Result<PackInfo, String> {
    modpack::inspect_modpack(path).await
}

#[tauri::command(rename_all = "snake_case")]
pub async fn prepare_online_modpack(
    client: tauri::State<'_, &MinecraftApiClient>,
    info: DownloadFile,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<OnlinePackInfo, String> {
    modpack::prepare_online_modpack(
        &client.downloader().await,
        info,
        task_id,
        super::channel_sink(on_event),
    )
    .await
}

#[tauri::command]
pub fn release_online_modpack(token: String) {
    modpack::release_online_modpack(token)
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)] // IPC 参数固定
pub async fn install_modpack(
    state: tauri::State<'_, Arc<AsyncMutex<AppState>>>,
    client: tauri::State<'_, &MinecraftApiClient>,
    path: PathBuf,
    instance_name: String,
    optional_files: Vec<String>,
    install_java: bool,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<(), String> {
    let repo = state.lock().await.active_repo_path.clone();
    let java = crate::config::java::installer_candidates(&state).await;
    modpack::install_modpack(
        &repo,
        &client,
        path,
        instance_name,
        optional_files,
        install_java,
        task_id,
        super::channel_sink(on_event),
        &java,
        &ConfigManager::instance().java_runtime_dir(),
        crate::assets::Assets::new(ConfigManager::instance().config_dir())
            .forge_installer()
            .as_deref(),
    )
    .await?;
    // 整合包可能包含加载器安装，完成后刷新一次 Java 列表。
    if let Err(error) = crate::config::java::refresh(true).await {
        log::warn!("Cannot refresh Java after modpack install: {error}");
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
#[allow(clippy::too_many_arguments)] // IPC 参数固定
pub async fn install_online_modpack(
    state: tauri::State<'_, Arc<AsyncMutex<AppState>>>,
    client: tauri::State<'_, &MinecraftApiClient>,
    token: String,
    instance_name: String,
    optional_files: Vec<String>,
    install_java: bool,
    task_id: i32,
    on_event: tauri::ipc::Channel<TaskItemReport>,
) -> Result<(), String> {
    let repo = state.lock().await.active_repo_path.clone();
    let java = crate::config::java::installer_candidates(&state).await;
    modpack::install_online_modpack(
        &repo,
        &client,
        token,
        instance_name,
        optional_files,
        install_java,
        task_id,
        super::channel_sink(on_event),
        &java,
        &ConfigManager::instance().java_runtime_dir(),
        crate::assets::Assets::new(ConfigManager::instance().config_dir())
            .forge_installer()
            .as_deref(),
    )
    .await?;
    // 整合包可能包含加载器安装，完成后刷新一次 Java 列表。
    if let Err(error) = crate::config::java::refresh(true).await {
        log::warn!("Cannot refresh Java after modpack install: {error}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn online_prepare_ipc_accepts_task_arguments_and_validates_source() {
        let client: &'static MinecraftApiClient = Box::leak(Box::new(MinecraftApiClient::new(
            pcl_core::io::net::http::HttpTransport::default(),
            &pcl_core::io::net::http::api_client::ApiProvider::Official,
        )));
        let app = tauri::test::mock_builder()
            .manage(client)
            .invoke_handler(tauri::generate_handler![prepare_online_modpack])
            .build(tauri::test::mock_context(tauri::test::noop_assets()))
            .unwrap();
        let window = tauri::WebviewWindowBuilder::new(&app, "main", Default::default())
            .build()
            .unwrap();
        let result = tauri::test::get_ipc_response(
            &window,
            tauri::webview::InvokeRequest {
                cmd: "prepare_online_modpack".into(),
                callback: tauri::ipc::CallbackFn(0),
                error: tauri::ipc::CallbackFn(1),
                url: window.url().unwrap(),
                body: tauri::ipc::InvokeBody::Json(serde_json::json!({
                    "info": { "url": "https://example.org/pack.mrpack", "size": 1, "sha1": "", "path": null },
                    "task_id": -2001, "version_id": "Pack", "on_event": "__CHANNEL__:1"
                })),
                headers: Default::default(),
                invoke_key: tauri::test::INVOKE_KEY.to_string(),
            },
        );
        assert_eq!(result.unwrap_err(), "Unsupported resource host");
    }
}
