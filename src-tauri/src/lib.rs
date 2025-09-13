use crate::core::java::JavaRuntimeVecExt;
use setup::AppState;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

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
            if let Some(config_manager) = setup::CONFIG_MANAGER.as_ref() {
                app.manage(Arc::clone(&config_manager.app_state));
            } else {
                log::error!("CONFIG_MANAGER is None");
                app.dialog()
                    .message("Config manager failed to initialize!")
                    .title("Fatal Error! 完🥚辣！")
                    .buttons(tauri_plugin_dialog::MessageDialogButtons::Ok)
                    .show(|_result| {
                        std::process::exit(1);
                    });
            }
            // let window = app.get_webview_window("main").unwrap();
            // window.on_navigation(move |url| {});
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
            commands::get_repositories,
            commands::get_account,
            commands::get_instances_in_repository,
            commands::select_instance,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
