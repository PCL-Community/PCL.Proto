use crate::config::{
    arguments::LaunchArguments, instance::InstanceConfig, launch_settings::InstanceLaunchSettings,
    manager::ConfigManager,
};

#[tauri::command]
pub async fn get_launch_arguments(
    instance_directory: Option<std::path::PathBuf>,
) -> Result<LaunchArguments, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let config = ConfigManager::instance();
        if let Some(path) = instance_directory {
            config.read_instance(&path, InstanceConfig::arguments)
        } else {
            Ok(config
                .app_state
                .blocking_lock()
                .launch_arguments
                .clone()
                .unwrap_or_else(LaunchArguments::global_default))
        }
    })
    .await
    .map_err(|error| error.to_string())?
}
#[tauri::command]
pub async fn set_launch_arguments(
    instance_directory: Option<std::path::PathBuf>,
    settings: LaunchArguments,
) -> Result<LaunchArguments, String> {
    tauri::async_runtime::spawn_blocking(move || {
        settings.validate(instance_directory.is_some())?;
        let config = ConfigManager::instance();
        if let Some(path) = instance_directory {
            config.update_instance(&path, |file| {
                file.set_arguments(&settings)?;
                Ok(settings)
            })
        } else {
            config.update(|state| {
                state.launch_arguments = Some(settings.clone());
                Ok(settings)
            })
        }
    })
    .await
    .map_err(|error| error.to_string())?
}
#[tauri::command]
pub async fn get_instance_launch_settings(
    instance_directory: std::path::PathBuf,
) -> Result<InstanceLaunchSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance()
            .read_instance(&instance_directory, InstanceConfig::launch_settings)
    })
    .await
    .map_err(|error| error.to_string())?
}
#[tauri::command]
pub async fn set_instance_launch_settings(
    instance_directory: std::path::PathBuf,
    settings: InstanceLaunchSettings,
) -> Result<InstanceLaunchSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update_instance(&instance_directory, |file| {
            file.set_launch_settings(&settings)?;
            Ok(settings)
        })
    })
    .await
    .map_err(|error| error.to_string())?
}
