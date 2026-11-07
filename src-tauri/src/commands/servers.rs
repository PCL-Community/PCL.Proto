use std::path::PathBuf;

use crate::commands::instance_resources::instance_game_directory;
use pcl_core::minecraft::servers;

#[tauri::command]
pub async fn list_servers(instance_directory: PathBuf) -> Result<servers::ServerList, String> {
    tauri::async_runtime::spawn_blocking(move || {
        servers::read(&instance_game_directory(&instance_directory)?)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn change_server(
    instance_directory: PathBuf,
    revision: String,
    change: servers::ServerChange,
) -> Result<servers::ServerList, String> {
    tauri::async_runtime::spawn_blocking(move || {
        servers::change(
            &instance_game_directory(&instance_directory)?,
            &revision,
            change,
        )
    })
    .await
    .map_err(|error| error.to_string())?
}
