//  ____   ____ _       ____            _
// |  _ \ / ___| |     |  _ \ _ __ ___ | |_ ___
// | |_) | |   | |     | |_) | '__/ _ \| __/ _ \
// |  __/| |___| |___ _|  __/| | | (_) | || (_) |
// |_|    \____|_____(_)_|   |_|  \___/ \__\___/
//
use core::downloader;
use core::java::JavaRuntimeVecExt;
use setup::AppState;
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{Target, TargetKind};

mod commands;
mod core;
mod scaffolding;
mod setup;
mod util;

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
                    // Target::new(TargetKind::Webview),
                ])
                .level(log::LevelFilter::Info)
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

            // 初始化scaffolding全局状态
            // let terracotta_state = Arc::new(tokio::sync::Mutex::new(TerracottaState::default()));
            // app.manage(terracotta_state);
            // app.manage(NetworkInstanceManager::default());

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
            core::api_client::fetch_with_modrinth,
            util::skin::fetch_skin_from_uuid_cached,
            util::skin::fetch_skin_from_url,
            // Scaffolding commands
            // scaffolding::commands::start_host, 
            // scaffolding::commands::start_guest,
            // scaffolding::commands::shutdown_room,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
