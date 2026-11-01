use crate::{
    AppState,
    app::config::ConfigManager,
    minecraft::instance::{McInstance, folder::McFolder},
};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub fn get_repositories(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<McFolder> {
    let state = state.blocking_lock();
    state.repositories.clone()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn select_repository(repository_index: usize) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().select_repository(repository_index)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_new_repository(
    new_repo_path: &str,
    new_repo_name: &str,
) -> Result<Vec<McFolder>, String> {
    ConfigManager::instance().add_repository(new_repo_name, std::path::Path::new(new_repo_path))
}
#[tauri::command(rename_all = "snake_case")]
pub async fn get_instances_in_repository(
    state: State<'_, Arc<Mutex<AppState>>>,
    repository_index: usize,
) -> Result<Vec<McInstance>, String> {
    let repo = state
        .lock()
        .await
        .repositories
        .get(repository_index)
        .cloned()
        .ok_or("Minecraft folder index is out of range")?;
    tauri::async_runtime::spawn_blocking(move || repo.game_instances())
        .await
        .map_err(|error| error.to_string())
}
