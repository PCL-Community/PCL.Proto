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

mod app;
mod commands;
mod io;
mod link;
mod minecraft;
mod protocol;
mod utils;

pub use app::config::{AppState, CONFIG_MANAGER, ConfigManager};

#[tauri::command]
async fn system_fonts() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        let mut names: Vec<_> = db
            .faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
            .collect();
        names.sort();
        names.dedup();
        names
    })
    .await
    .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    #[cfg(all(target_os = "macos", not(dev)))]
    let context = tauri::generate_context!(
        assets = app::resources::FrontendAssets::new()
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
            if let Ok(config_manager) = app::config::CONFIG_MANAGER.as_ref() {
                app.manage(Arc::clone(&config_manager.app_state));
                app.manage(&config_manager.api_client);
                terracotta::init_lib(config_manager.identifier_path());
            } else {
                log::error!("Config manager failed: {:?}", app::config::CONFIG_MANAGER.as_ref().err());
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
            if app::config::CONFIG_MANAGER.is_ok() {
                tauri::async_runtime::spawn(async move {
                    if let Err(error) = minecraft::java::refresh(false).await {
                        log::error!("Java scan failed: {error}");
                    }
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            system_fonts,
            commands::homepage::load_custom_homepage,
            commands::homepage::save_custom_homepage,
            commands::launch::begin_launch,
            commands::launch::cancel_launch,
            commands::launch::launch_game,
            minecraft::launch::session::get_game_sessions,
            minecraft::launch::session::stop_game_session,
            minecraft::launch::session::clear_game_session_log,
            io::download::resources::get_download_stats,
            io::download::resources::set_download_settings,
            io::download::resources::get_download_settings,
            app::config::get_log_max_lines,
            app::config::set_log_max_lines,
            app::config::get_window_settings,
            app::config::set_window_settings,
            app::config::get_launcher_visibility,
            app::config::set_launcher_visibility,
            app::config::get_version_info,
            app::config::set_version_info,
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
            commands::instance_resources::instance_resource_destination,
            minecraft::export::resource::download_resource,
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
            utils::toys::get_lucky_today,
            utils::external::open_external_url,
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
            minecraft::export::version::export_minecraft_version,
            minecraft::export::version::open_minecraft_wiki,
            minecraft::export::loader::save_loader_file,
            minecraft::export::loader::open_loader_changelog,
            io::download::download_custom_file,
            io::download::tasks::prepare_download_task,
            io::download::tasks::cancel_download_task,
            io::download::tasks::release_download_task,
            io::download::download_fabric_installer,
            io::download::open_custom_download_folder,
            minecraft::installer::download_minecraft_version,
            minecraft::installer::forgelike::prepare_forgelike_java,
            minecraft::installer::forgelike::forgelike_installer_name,
            minecraft::installer::forgelike::download_forgelike_installer,
            minecraft::installer::modpack::inspect_modpack,
            minecraft::installer::modpack::install_modpack,
            minecraft::installer::modpack::prepare_online_modpack,
            minecraft::installer::modpack::release_online_modpack,
            minecraft::installer::modpack::install_online_modpack,
            commands::versions::get_plugin_versions,
            commands::versions::get_fabric_installers,
            commands::instances::get_active_instance,
            link::mcping::server_query,
            commands::skin::fetch_username_uuid,
            commands::skin::fetch_uuid_profile,
            io::net::http::api_client::fetch_with_modrinth,
            io::net::http::api_client::fetch_with_curseforge,
            io::net::http::api_client::curseforge_available,
            commands::skin::fetch_skin_from_uuid_cached,
            commands::skin::fetch_skin_from_url,
            commands::skin::fetch_profile_skin,
            commands::skin::change_offline_skin,
            commands::skin::save_profile_skin,
            commands::skin::change_microsoft_skin,
            // scaffolding
            link::scaffolding::get_terracotta_meta,
            link::scaffolding::get_terracotta_state,
            link::scaffolding::set_terracotta_waiting,
            link::scaffolding::set_terracotta_host_scanning,
            link::scaffolding::set_terracotta_host_starting,
            link::scaffolding::set_terracotta_guesting,
        ])
        .run(context)
        .expect("error while running tauri application");
}
