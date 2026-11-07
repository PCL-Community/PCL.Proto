use crate::config::manager::ConfigManager;

#[tauri::command]
pub async fn get_proxy_settings() -> crate::config::network::ProxySettings {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .proxy
        .clone()
}

#[tauri::command]
pub async fn set_proxy_settings(
    settings: crate::config::network::ProxySettings,
) -> Result<crate::config::network::ProxySettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update_proxy_settings(settings)
    })
    .await
    .map_err(|error| error.to_string())?
}
