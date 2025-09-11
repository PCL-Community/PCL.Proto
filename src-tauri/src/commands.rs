use std::{
    ops::Deref,
    sync::{Arc, Mutex},
};

use crate::{
    AppState,
    core::{
        auth::Account,
        java::{JavaRuntime, JavaRuntimeVecExt},
        launcher::LaunchOption,
    },
    setup::GameRepository,
};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn launch_game(app: AppHandle, state: State<'_, Arc<Mutex<AppState>>>) -> Result<(), String> {
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
    return Ok(java_runtimes);
}

#[tauri::command]
pub fn get_game_directories(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<GameRepository> {
    let state = state.lock().unwrap();
    state.game_directories.clone()
}

#[tauri::command]
pub fn get_account(state: State<'_, Arc<Mutex<AppState>>>) -> Account {
    let state = state.lock().unwrap();
    state.active_account.as_ref().clone()
}
