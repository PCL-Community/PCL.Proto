//! 启动器设置类命令：窗口、可见性、版本说明和日志行数。

use crate::config::WindowSettings;
use crate::config::manager as settings;
use crate::window::LauncherVisibility;

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
