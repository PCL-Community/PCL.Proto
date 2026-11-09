//  ____   ____ _       ____            _
// |  _ \ / ___| |     |  _ \ _ __ ___ | |_ ___
// | |_) | |   | |     | |_) | '__/ _ \| __/ _ \
// |  __/| |___| |___ _|  __/| | | (_) | || (_) |
// |_|    \____|_____(_)_|   |_|  \___/ \__\___/
//
use std::sync::Arc;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_log::{Target, TargetKind};

mod assets;
mod commands;
mod config;
mod external;
mod identity;
mod link;
mod login;
mod protocol;
mod resources;
mod secrets;
mod window;

pub use config::manager::{AppState, CONFIG_MANAGER, ConfigManager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(all(target_os = "macos", not(dev)))]
    let context = tauri::generate_context!(
        assets = resources::FrontendAssets::new()
            .expect("could not load frontend resources from the .app bundle")
    );
    #[cfg(not(all(target_os = "macos", not(dev))))]
    let context = tauri::generate_context!();

    tauri::Builder::default()
        .register_uri_scheme_protocol("pclinstance", |_app, request| protocol::serve(request))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_os::init())
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    // CE 同款：每次启动写一个 `Launch-yyyy-M-d-HHmmssfff.log`，超 32MB 归档。
                    Target::new(TargetKind::LogDir {
                        file_name: Some(commands::logs::launch_log_stem()),
                    }),
                    // Target::new(TargetKind::Webview),
                ])
                .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
                .max_file_size(32 * 1024 * 1024)
                .level(log::LevelFilter::Info)
                .build(),
        )
        .setup(|app| {
            if let Ok(config_manager) = crate::config::manager::CONFIG_MANAGER.as_ref() {
                app.manage(Arc::clone(&config_manager.app_state));
                app.manage(&config_manager.api_client);
                // 联机机器码独立存放，避免 launcher id 文件长度变化时被 Terracotta 覆写。
                pcl_core::link::init(crate::config::link::machine_id_path(config_manager.config_dir()));
            } else {
                log::error!("Config manager failed: {:?}", crate::config::manager::CONFIG_MANAGER.as_ref().err());
                app.dialog()
                    .message("Could not load the launcher configuration. The existing configuration has not been overwritten. Check the logs before restarting.")
                    .title("Fatal Error! 完🥚辣！")
                    .buttons(tauri_plugin_dialog::MessageDialogButtons::Ok)
                    .show(|_result| {
                        std::process::exit(1);
                    });
            }
            // let easytier_manager = easytier::instance_manager::NetworkInstanceManager::new();
            // app.manage(easytier_manager);
            // let window = app.get_webview_window("main").unwrap();
            // window.on_navigation(move |url| {});
            // search for Java during init
            if crate::config::manager::CONFIG_MANAGER.is_ok() {
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = config::java::refresh(false).await {
                        log::error!("Java scan failed: {error}");
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::system::system_fonts,
            commands::title_logo::get_title_logo,
            commands::title_logo::set_title_logo,
            commands::title_logo::clear_title_logo,
            commands::homepage::load_custom_homepage,
            commands::homepage::save_custom_homepage,
            commands::launch::begin_launch,
            commands::launch::cancel_launch,
            commands::launch::launch_game,
            commands::session::get_game_sessions,
            commands::session::stop_game_session,
            commands::session::clear_game_session_log,
            commands::download::get_download_stats,
            commands::download::set_download_settings,
            commands::download::get_download_settings,
            commands::config::get_log_max_lines,
            commands::config::set_log_max_lines,
            commands::config::get_window_settings,
            commands::config::set_window_settings,
            commands::config::get_launcher_visibility,
            commands::config::set_launcher_visibility,
            commands::config::get_version_info,
            commands::config::set_version_info,
            commands::memory::get_memory_settings,
            commands::memory::set_memory_settings,
            commands::memory::preview_memory_settings,
            commands::instances::get_isolation_default,
            commands::instances::set_isolation_default,
            commands::instances::get_instance_isolation,
            commands::instances::set_instance_isolation,
            commands::launch_settings::get_launch_arguments,
            commands::launch_settings::set_launch_arguments,
            commands::launch_settings::get_instance_launch_settings,
            commands::launch_settings::set_instance_launch_settings,
            commands::network::get_proxy_settings,
            commands::network::set_proxy_settings,
            commands::logs::list_log_files,
            commands::logs::open_log_directory,
            commands::logs::open_log_path,
            commands::logs::log_export_destination,
            commands::logs::export_log_files,
            commands::logs::clear_log_files,
            commands::launch::export_launch_script,
            commands::instance_resources::repair_instance_files,
            commands::instances::get_instance_description,
            commands::instances::get_instance_launch_count,
            commands::instance_resources::instance_resource_destination,
            commands::export::download_resource,
            commands::instances::set_instance_description,
            commands::instances::rename_instance,
            commands::instance_resources::list_instance_resources,
            commands::instance_resources::get_save_info,
            commands::instance_resources::update_save_info,
            commands::servers::list_servers,
            commands::servers::change_server,
            commands::instance_resources::open_instance_resource_folder,
            commands::instance_resources::open_instance_path,
            commands::instance_resources::set_instance_resources_enabled,
            commands::instance_resources::delete_instance_resources,
            commands::instance_resources::import_instance_resources,
            commands::system::get_lucky_today,
            commands::system::open_external_url,
            commands::java::add_java,
            commands::java::get_java_list,
            commands::java::refresh_java_list,
            commands::java::get_java_settings,
            commands::java::set_global_java,
            commands::java::set_java_enabled,
            commands::java::set_instance_java_settings,
            commands::java::select_relative_java,
            commands::java::open_java_folder,
            commands::instances::open_instance_folder,
            commands::java::prepare_game_java,
            commands::java::download_game_java,
            commands::java::cancel_java_download,
            commands::repositories::get_repositories,
            commands::repositories::select_repository,
            commands::repositories::add_new_repository,
            commands::account::get_profiles,
            commands::account::select_profile,
            commands::account::delete_profile,
            commands::account::create_offline_profile,
            commands::account::set_offline_uuid,
            commands::account::rename_microsoft_profile,
            commands::account::probe_auth_server,
            commands::account::start_login,
            commands::account::login_state,
            commands::account::cancel_login,
            commands::account::select_login_profile,
            commands::repositories::get_instances_in_repository,
            commands::instances::instance_folder_action,
            commands::instances::select_instance,
            commands::versions::get_version_manifest,
            commands::versions::handle_clicked_on_version,
            commands::export::export_minecraft_version,
            commands::export::open_minecraft_wiki,
            commands::export::save_loader_file,
            commands::export::open_loader_changelog,
            commands::download::download_custom_file,
            commands::download::prepare_download_task,
            commands::download::cancel_download_task,
            commands::download::release_download_task,
            commands::download::download_fabric_installer,
            commands::download::open_custom_download_folder,
            commands::installer::download_minecraft_version,
            commands::installer::prepare_forgelike_java,
            commands::installer::forgelike_installer_name,
            commands::installer::download_forgelike_installer,
            commands::installer::inspect_modpack,
            commands::installer::install_modpack,
            commands::installer::prepare_online_modpack,
            commands::installer::release_online_modpack,
            commands::installer::install_online_modpack,
            commands::versions::get_plugin_versions,
            commands::versions::get_fabric_installers,
            commands::instances::get_active_instance,
            commands::system::server_query,
            commands::skin::fetch_username_uuid,
            commands::skin::fetch_skin_url,
            commands::sources::fetch_with_modrinth,
            commands::sources::fetch_with_curseforge,
            commands::sources::curseforge_available,
            commands::skin::fetch_skin_from_uuid_cached,
            commands::skin::fetch_skin_from_url,
            commands::skin::fetch_profile_skin,
            commands::skin::change_offline_skin,
            commands::skin::save_profile_skin,
            commands::skin::change_microsoft_skin,
            // scaffolding
            commands::link::initialize_link,
            commands::link::get_link_settings,
            commands::link::set_link_settings,
            commands::link::reset_link_settings,
            commands::link::set_link_eula,
            commands::link::refresh_link_announcement,
            commands::link::get_link_account,
            commands::link::start_link_naid_login,
            commands::link::logout_link_naid,
            commands::link::refresh_link_naid,
            commands::link::test_link_nat,
            commands::link::discover_link_worlds,
            commands::link::verify_link_port,
            commands::link::get_link_lobby_state,
            commands::link::lobby_create,
            commands::link::lobby_join,
            commands::link::lobby_leave,
            commands::link::lobby_reset,
            commands::link_runtime::get_link_runtime_status,
            commands::link_runtime::install_link_runtime,
        ])
        .run(context)
        .expect("error while running tauri application");
}
