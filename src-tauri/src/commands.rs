use crate::{
    AppState,
    core::{
        api_client::{self, plugins::McPluginReport},
        auth::Account,
        game::GameInstance,
        java::{JavaRuntime, JavaRuntimeVecExt},
        launcher::LaunchOption,
        mcmod,
        repository::GameRepository,
    },
    setup::ConfigManager,
};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::Mutex;

#[tauri::command]
pub fn launch_game(_app: AppHandle, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    log::info!("launch_game invoked from js.");
    let guard = state.blocking_lock();
    let launch_option = LaunchOption::from_state(&guard);
    drop(guard);
    if let Err(e) = launch_option {
        log::error!("launch_game: {:?}", e);
        return Err(e.to_string());
    } else {
        if let Err(e) = launch_option.unwrap().launch_checked() {
            log::error!("launch_game: {:?}", e);
            return Err(e.to_string());
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn add_java(
    app: AppHandle,
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<JavaRuntime, ()> {
    if let Some(file_path) = app.dialog().file().blocking_pick_file() {
        let file_path_str = file_path.to_string();
        if let Ok(mut java_runtime) = JavaRuntime::try_from(file_path_str.as_str()) {
            java_runtime.is_user_imported = true;
            let mut state = state.lock().await;
            if state.java_runtimes.contains(&java_runtime) {
                return Err(());
            }
            state.java_runtimes.push(java_runtime.clone());
            log::info!("add_java: {:?}", java_runtime.java_exe);
            return Ok(java_runtime);
        } else {
            return Err(());
        }
    } else {
        Err(())
    }
}

#[tauri::command]
pub fn get_java_list(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<JavaRuntime> {
    let state = state.blocking_lock();
    state.java_runtimes.clone()
}

#[tauri::command]
pub async fn refresh_java_list() -> Result<Vec<JavaRuntime>, ()> {
    let java_runtimes = JavaRuntime::search().await;
    let return_runtimes = java_runtimes.clone();
    java_runtimes.patch_state().await;
    Ok(return_runtimes)
}

#[tauri::command]
pub fn get_repositories(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<GameRepository> {
    let state = state.blocking_lock();
    state.repositories.clone()
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_new_repository(
    state: State<'_, Arc<Mutex<AppState>>>,
    new_repo_path: &str,
    new_repo_name: &str,
) -> Result<Vec<GameRepository>, String> {
    let mut guard = state.blocking_lock();
    if guard.repositories.iter().any(|repo_old| {
        repo_old.name == new_repo_name || repo_old.path.as_os_str() == new_repo_path
    }) {
        return Err("repo info conflits with current!".into());
    }
    guard
        .repositories
        .push(GameRepository::new(new_repo_name, new_repo_path.into()));
    log::info!(
        "added new repo, name: {}, path: {}, repos count now: {}",
        new_repo_name,
        new_repo_path,
        guard.repositories.len()
    );
    Ok(guard.repositories.clone())
}

/// async commands that contain references as inputs must return a Result
/// so this place returns an option
#[tauri::command]
pub async fn get_account() -> Option<Account> {
    let account = &ConfigManager::instance()
        .app_state
        .lock()
        .await
        .active_account;
    account.as_deref().cloned()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_instances_in_repository(
    state: State<'_, Arc<Mutex<AppState>>>,
    repository_index: usize,
) -> Result<Vec<GameInstance>, ()> {
    let state = state.lock().await;
    let all_repos = &state.repositories;
    let repo = &all_repos[repository_index];
    Ok(repo.game_instances().to_owned())
}

#[tauri::command(rename_all = "snake_case")]
pub fn get_active_instance(state: State<'_, Arc<Mutex<AppState>>>) -> Option<GameInstance> {
    let guard = state.blocking_lock();
    let active_instance = guard.active_game_instance.clone();
    active_instance.as_deref().cloned()
}

#[tauri::command(rename_all = "snake_case")]
pub fn select_instance(
    state: State<'_, Arc<Mutex<AppState>>>,
    repository_index: usize,
    instance_id: &str,
) {
    let mut guard = state.blocking_lock();
    let instances = guard.repositories[repository_index].game_instances();
    let instance = instances
        .iter()
        .find(|instance| instance.id == instance_id)
        .unwrap();
    guard.active_game_instance = Some(Arc::new(instance.clone()));
}

#[tauri::command]
pub async fn get_version_manifest(
    client: State<'_, &api_client::MinecraftApiClient>,
) -> Result<api_client::game::VersionManifest, String> {
    match client
        .get_with_endpoint::<api_client::game::VersionManifest>(
            api_client::game::VERSION_MANIFEST_ENDPOINT,
            true,
        )
        .await
    {
        Ok(manifest) => Ok(manifest),
        Err(e) => return Err(format!("get_version_manifest: {:?}", e)),
    }
}

/// return the version json
#[tauri::command]
pub async fn handle_clicked_on_version(
    client: State<'_, &api_client::MinecraftApiClient>,
    id: &str,
) -> Result<api_client::game::VersionDetails, String> {
    let temp_dir = std::env::temp_dir().join(format!("pcl-proto-{}", id));
    log::debug!("selected tmp: {:?}", &temp_dir);
    let version_detail = client
        .get_version_details(id, &temp_dir)
        .await
        .map_err(|err| err.to_string())?;
    Ok(version_detail)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_plugin_versions(
    api_client: State<'_, &api_client::MinecraftApiClient>,
    plugin_type: mcmod::PluginType,
    mc_version: &str,
) -> Result<Vec<McPluginReport>, String> {
    let verisons = match plugin_type {
        mcmod::PluginType::Forge => api_client.get_forge_versions(mc_version).await,
        mcmod::PluginType::Fabric => api_client.get_fabric_versions(mc_version).await,
        _ => Err(api_client::McApiError::PluginMismatch(plugin_type)),
    };
    verisons.map_err(|err| err.to_string())
}

#[tauri::command]
pub async fn fetch_with_modrinth(
    api_client: State<'_, &api_client::MinecraftApiClient>,
    endpoint: &str,
) -> Result<serde_json::Value, String> {
    api_client.fetch_with_modrinth(endpoint).await.map_err(|err| err.to_string())
}
