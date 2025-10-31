use crate::core::java::JavaRuntimeVecExt;
use core::downloader;
use setup::AppState;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{Target, TargetKind};

mod commands;
mod core;
mod setup;
mod util;

// #[cfg(not(target_os = "android"))]
// fn toggle_window_visibility<R: tauri::Runtime>(app: &tauri::AppHandle<R>) {
//     if let Some(window) = app.get_webview_window("main") {
//         if window.is_visible().unwrap_or_default() {
//             if window.is_minimized().unwrap_or_default() {
//                 let _ = window.unminimize();
//                 let _ = window.set_focus();
//             } else {
//                 let _ = window.hide();
//             }
//         } else {
//             let _ = window.show();
//             let _ = window.set_focus();
//         }
//     }
// }

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                    Target::new(TargetKind::Webview),
                ])
                .level(log::LevelFilter::Debug)
                .build(),
        )
        .setup(|app| {
            if let Some(config_manager) = setup::CONFIG_MANAGER.as_ref() {
                app.manage(Arc::clone(&config_manager.app_state));
                app.manage(&config_manager.api_client);
                log::debug!("app state managed");
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
                java_runtimes.patch_state().await;
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
            commands::add_new_repository,
            commands::get_account,
            commands::get_instances_in_repository,
            commands::select_instance,
            commands::get_version_manifest,
            commands::handle_clicked_on_version,
            downloader::minecraft_resource::download_minecraft_version,
            commands::get_plugin_versions,
            commands::get_active_instance,
            util::server_query::server_query,
            util::skin::fetch_username_uuid,
            util::skin::fetch_uuid_profile,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
