use std::sync::{Arc, Mutex};

use crate::{
    AppState,
    core::{auth::Account, java::JavaRuntime},
    setup::GameDir,
};
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_dialog::DialogExt;

#[tauri::command]
pub fn launch_game(app: AppHandle) {
    println!("[game] launch_game invoked from js.");
    app.emit(
        "modal-open",
        "Game launching feature is not implemented yet!",
    )
    .unwrap();
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
pub async fn refresh_java_list(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<JavaRuntime>, ()> {
    let java_runtimes = JavaRuntime::search().await;
    let mut state = state.lock().unwrap();
    state.java_runtimes = java_runtimes.clone();
    return Ok(java_runtimes);
}

#[tauri::command]
pub fn get_game_directories(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<GameDir> {
    let state = state.lock().unwrap();
    state.game_directories.clone()
}

#[tauri::command]
pub fn get_account(state: State<'_, Arc<Mutex<AppState>>>) -> Account {
    let state = state.lock().unwrap();
    state.account.clone()
}
