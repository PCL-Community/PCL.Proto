use crate::{app::config::ConfigManager, minecraft::java::preference::current_instance};

#[tauri::command]
pub async fn get_launch_arguments(
    instance_directory: Option<std::path::PathBuf>,
) -> Result<crate::minecraft::launch::arguments::LaunchArguments, String> {
    let state = ConfigManager::instance().app_state.lock().await;
    if let Some(path) = instance_directory {
        current_instance(&state, &path).map_err(|error| error.to_string())?;
        Ok(state
            .instance_launch_arguments
            .get(&path)
            .cloned()
            .unwrap_or_default())
    } else {
        Ok(state
            .launch_arguments
            .clone()
            .unwrap_or_else(crate::minecraft::launch::arguments::LaunchArguments::global_default))
    }
}

#[tauri::command]
pub async fn set_launch_arguments(
    instance_directory: Option<std::path::PathBuf>,
    settings: crate::minecraft::launch::arguments::LaunchArguments,
) -> Result<crate::minecraft::launch::arguments::LaunchArguments, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            crate::minecraft::launch::arguments::update(
                state,
                instance_directory.as_deref(),
                settings,
            )
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_instance_launch_settings(
    instance_directory: std::path::PathBuf,
) -> Result<crate::minecraft::launch::settings::InstanceLaunchSettings, String> {
    let state = ConfigManager::instance().app_state.lock().await;
    current_instance(&state, &instance_directory).map_err(|error| error.to_string())?;
    Ok(state
        .instance_launch_settings
        .get(&instance_directory)
        .cloned()
        .unwrap_or_default())
}

#[tauri::command]
pub async fn set_instance_launch_settings(
    instance_directory: std::path::PathBuf,
    settings: crate::minecraft::launch::settings::InstanceLaunchSettings,
) -> Result<crate::minecraft::launch::settings::InstanceLaunchSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            crate::minecraft::launch::settings::update(state, &instance_directory, settings)
        })
    })
    .await
    .map_err(|error| error.to_string())?
}
