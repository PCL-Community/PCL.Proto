//! 启动器设置类命令：窗口、可见性、版本说明和日志行数。

use crate::config::WindowSettings;
use crate::config::launch_settings::GlobalLaunchPreferences;
use crate::config::manager as settings;
use crate::window::LauncherVisibility;

#[tauri::command]
pub async fn get_launch_preferences() -> GlobalLaunchPreferences {
    settings::get_launch_preferences().await
}

#[tauri::command]
pub async fn set_launch_preferences(
    preferences: GlobalLaunchPreferences,
) -> Result<GlobalLaunchPreferences, String> {
    settings::set_launch_preferences(preferences).await
}

#[tauri::command]
pub async fn get_window_settings() -> Option<WindowSettings> {
    settings::get_window_settings().await
}

#[tauri::command]
pub async fn set_window_settings(settings: WindowSettings) -> Result<WindowSettings, String> {
    settings::set_window_settings(settings).await
}

#[tauri::command]
pub async fn get_launcher_visibility() -> LauncherVisibility {
    settings::get_launcher_visibility().await
}

#[tauri::command]
pub async fn set_launcher_visibility(mode: LauncherVisibility) -> Result<(), String> {
    settings::set_launcher_visibility(mode).await
}

#[tauri::command]
pub async fn get_version_info() -> String {
    settings::get_version_info().await
}

#[tauri::command]
pub async fn set_version_info(value: String) -> Result<String, String> {
    settings::set_version_info(value).await
}

#[tauri::command]
pub async fn get_log_max_lines() -> Option<u32> {
    settings::get_log_max_lines().await
}

#[tauri::command]
pub async fn set_log_max_lines(lines: u32) -> Result<u32, String> {
    settings::set_log_max_lines(lines).await
}

/// 读取用户放在配置目录下的 `hints.txt`（CE 的 `PCL/hints.txt`），用于启动页「你知道吗」。
#[tauri::command]
pub async fn read_launch_hints() -> Option<String> {
    let path = crate::config::manager::ConfigManager::instance()
        .config_dir()
        .join("hints.txt");
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path).ok()?;
        Some(String::from_utf8_lossy(crate::config::strip_bom(&bytes)).into_owned())
    })
    .await
    .ok()
    .flatten()
}

/// 导出整份 `config.yml`，与 CE「导出设置」一致。
#[tauri::command]
pub async fn export_launcher_settings(path: std::path::PathBuf) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let source = crate::config::manager::ConfigManager::instance().config_path();
        std::fs::copy(source, &path).map(|_| ()).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

/// 覆盖当前 `config.yml` 并重启启动器，与 CE「导入设置」一致。
#[tauri::command]
pub async fn import_launcher_settings(
    app: tauri::AppHandle,
    path: std::path::PathBuf,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let bytes = std::fs::read(&path).map_err(|error| error.to_string())?;
        let state: crate::config::manager::AppState =
            serde_yaml::from_slice(crate::config::strip_bom(&bytes))
                .map_err(|error| format!("Invalid launcher configuration: {error}"))?;
        if !crate::config::manager::valid_loaded_state(&state) {
            return Err("Invalid launcher configuration".into());
        }
        std::fs::copy(&path, crate::config::manager::ConfigManager::instance().config_path())
            .map(|_| ())
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())??;
    app.restart()
}
