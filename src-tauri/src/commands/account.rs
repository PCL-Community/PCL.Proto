use crate::{app::config::ConfigManager, minecraft::profile::SafeProfile};

/// async commands that contain references as inputs must return a Result
/// so this place returns an option
#[tauri::command]
pub async fn get_account() -> Option<SafeProfile> {
    let account = &ConfigManager::instance()
        .app_state
        .lock()
        .await
        .active_account;
    account.as_deref().map(SafeProfile::from)
}
