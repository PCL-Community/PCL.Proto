use std::sync::{Arc, Mutex};

use crate::{
    AppState,
    core::{
        api_client,
        auth::Account,
        game::GameInstance,
        java::{JavaRuntime, JavaRuntimeVecExt},
        launcher::LaunchOption,
        repository::GameRepository,
    },
    setup::ConfigManager,
};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn launch_game(_app: AppHandle, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
    log::info!("launch_game invoked from js.");
    let guard = state.lock().unwrap();
    let launch_option = LaunchOption::from_state(&guard);
    drop(guard);
    if let Err(e) = launch_option {
        log::error!("launch_game: {:?}", e);
        return Err(e.to_string());
    } else {
        if let Err(e) = launch_option.unwrap().launch() {
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
            let mut state = state.lock().unwrap();
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
    let state = state.lock().unwrap();
    state.java_runtimes.clone()
}

#[tauri::command]
pub async fn refresh_java_list() -> Result<Vec<JavaRuntime>, ()> {
    let java_runtimes = JavaRuntime::search().await;
    java_runtimes.clone().patch_state();
    Ok(java_runtimes)
}

#[tauri::command]
pub fn get_repositories(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<GameRepository> {
    let state = state.lock().unwrap();
    state.repositories.clone()
}

#[tauri::command]
pub fn get_account(state: State<'_, Arc<Mutex<AppState>>>) -> Option<Account> {
    let state = state.lock().unwrap();
    state.active_account.as_deref().cloned()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_instances_in_repository(
    state: State<'_, Arc<Mutex<AppState>>>,
    repository_name: &str,
) -> Result<Vec<GameInstance>, ()> {
    let state = state.lock().unwrap();
    let all_repos = &state.repositories;
    let repo = all_repos.iter().find(|repo| repo.name == repository_name);
    match repo {
        Some(repo) => Ok(repo.game_instances().to_vec()),
        None => Err(()),
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn select_instance(
    state: State<'_, Arc<Mutex<AppState>>>,
    repository_name: &str,
    instance_id: &str,
) -> Result<(), ()> {
    let mut state = state.lock().unwrap();
    let instances = state
        .repositories
        .iter()
        .find(|repo| repo.name == repository_name)
        .ok_or(())?
        .game_instances();
    let instance = instances
        .iter()
        .find(|instance| instance.id == instance_id)
        .ok_or(())?;
    state.active_game_instance = Some(Arc::new(instance.clone()));
    Ok(())
}

#[tauri::command]
pub async fn get_version_manifest() -> Result<api_client::game::VersionManifest, String> {
    let client = &ConfigManager::instance().api_client;
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

#[tauri::command]
pub async fn handle_clicked_on_version(id: &str) -> Result<bool, String> {
    // TODO: should be managed by task manager
    // STEP1: get the version json
    let client = &ConfigManager::instance().api_client;
    let temp_dir = std::env::temp_dir().join(format!("pcl-proto-{}", id));
    let version_detail = client
        .get_version_details(id, &temp_dir)
        .await
        .map_err(|err| err.to_string())?;
    // STEP2: start a task of downloading the version jar

    // STEP3: download libraries
    Ok(true)
}
