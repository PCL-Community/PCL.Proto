use crate::core::java::JavaRuntimeVecExt;
use setup::AppState;
use std::sync::Arc;
use tauri::Manager;

mod commands;
mod core;
mod setup;
mod util;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            app.manage(Arc::clone(&setup::ConfigManager::instance().app_state));
            // search for Java during init
            tauri::async_runtime::spawn(async move {
                let java_runtimes = core::java::JavaRuntime::search().await;
                java_runtimes.patch_state();
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::launch_game,
            util::toys::get_lucky_today,
            commands::add_java,
            commands::get_java_list,
            commands::refresh_java_list,
            commands::get_game_directories,
            commands::get_account
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
