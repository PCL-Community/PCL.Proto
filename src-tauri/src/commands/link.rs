//! 联机（工具-联机页 / 设置-联机页）IPC 命令。

use crate::config::link::LinkSettings;
use crate::link::LinkError;
use crate::link::announcement::LinkAnnouncement;
use crate::link::naid::{self, LinkAccount};
use crate::link::service::{self, LinkInit, LinkLobbyState, LinkNatStatus, LinkWorld};

#[tauri::command]
pub async fn initialize_link() -> Result<LinkInit, LinkError> {
    service::initialize().await
}

#[tauri::command]
pub async fn get_link_settings() -> LinkSettings {
    service::settings().await
}

#[tauri::command]
pub async fn set_link_settings(settings: LinkSettings) -> Result<LinkSettings, LinkError> {
    service::update_settings(settings).await
}

#[tauri::command]
pub async fn reset_link_settings() -> Result<LinkSettings, LinkError> {
    service::reset_settings().await
}

#[tauri::command]
pub async fn set_link_eula(accepted: bool) -> Result<bool, LinkError> {
    service::set_eula(accepted).await
}

#[tauri::command]
pub async fn refresh_link_announcement() -> Result<LinkAnnouncement, LinkError> {
    service::refresh_announcement().await
}

#[tauri::command]
pub async fn get_link_account() -> LinkAccount {
    naid::account().await
}

#[tauri::command]
pub async fn start_link_naid_login() -> Result<LinkAccount, LinkError> {
    let _control = service::CONTROL.lock().await;
    naid::start_login().await
}

#[tauri::command]
pub async fn logout_link_naid() -> Result<LinkAccount, LinkError> {
    let _control = service::CONTROL.lock().await;
    naid::logout().await
}

#[tauri::command]
pub async fn refresh_link_naid() -> Result<LinkAccount, LinkError> {
    let _control = service::CONTROL.lock().await;
    naid::refresh().await
}

#[tauri::command]
pub async fn test_link_nat() -> Result<LinkNatStatus, LinkError> {
    service::nat_status().await
}

#[tauri::command]
pub async fn discover_link_worlds() -> Result<Vec<LinkWorld>, LinkError> {
    service::discover_worlds().await
}

#[tauri::command]
pub async fn verify_link_port(port: i64) -> Result<LinkWorld, LinkError> {
    service::verify_port(port).await
}

#[tauri::command]
pub async fn get_link_lobby_state() -> Result<LinkLobbyState, LinkError> {
    service::lobby_state().await
}

#[tauri::command]
pub async fn lobby_create(port: i64) -> Result<LinkLobbyState, LinkError> {
    service::create(port).await
}

#[tauri::command]
pub async fn lobby_join(code: String) -> Result<LinkLobbyState, LinkError> {
    service::join(code).await
}

#[tauri::command]
pub async fn lobby_leave() -> Result<LinkLobbyState, LinkError> {
    service::stop().await
}

#[tauri::command]
pub async fn lobby_reset() -> Result<LinkLobbyState, LinkError> {
    service::stop().await
}
