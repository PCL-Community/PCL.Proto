use super::terracotta::room::RoomCode;
use crate::scaffolding::terracotta::room::PUBLIC_SERVERS;
use crate::scaffolding::terracotta::states::TerracottaState;
use easytier::common::config::ConfigFileControl;
use easytier::instance_manager::NetworkInstanceManager;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// 启动联机中心
#[tauri::command]
pub async fn start_host(
    terracotta_state: State<'_, Arc<Mutex<TerracottaState>>>,
    instance_manager: State<'_, NetworkInstanceManager>,
    player_name: &str,
    port: u16,
) -> Result<String, String> {
    log::info!("Starting host with player: {}, port: {}", player_name, port);
    let room_code = Arc::new(RoomCode::generate());
    log::info!("Generated room code: {}", room_code.code);
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::HostStarting {
            room: room_code.clone(),
            port,
        };
        drop(state);
    }
    log::info!(
        "Attempting to start room host with public servers: {:?}",
        PUBLIC_SERVERS
    );
    let config = room_code.compute_arguments_host(port, PUBLIC_SERVERS);
    let uuid = instance_manager.run_network_instance(config, true, ConfigFileControl::STATIC_CONFIG).map_err(|e| {
        log::error!("Failed to start room host: {}", e);
        e.to_string()
    })?;
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::HostOk {
            room: room_code.clone(),
            port,
            easytier: uuid,
            player_profiles: Vec::new(),
        };
    }
    log::info!(
        "Room host started successfully with code: {}",
        room_code.code
    );
    Ok(room_code.code.clone())
}

/// 加入联机
#[tauri::command]
pub async fn start_guest(
    terracotta_state: State<'_, Arc<tokio::sync::Mutex<TerracottaState>>>,
    code: &str,
    player_name: &str,
) -> Result<(), String> {
    log::info!(
        "Starting guest with room code: {}, player: {}",
        code,
        player_name
    );
    // 解析房间码
    let room_code = RoomCode::from_code(code).ok_or("Invalid room code".to_string())?;
    let room_code = Arc::new(room_code);
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::GuestConnecting {
            room: room_code.clone(),
        };
        drop(state);
    }
    log::info!(
        "Attempting to join room as guest with public servers: {:?}",
        PUBLIC_SERVERS
    );
    // room_code.compute_arguments_guest(Some(player_name), PUBLIC_SERVERS);
    log::info!("Guest connection initiated successfully");
    Ok(())
}

#[tauri::command]
pub async fn shutdown_room(
    terracotta_state: State<'_, Arc<tokio::sync::Mutex<TerracottaState>>>,
) -> Result<String, String> {
    log::info!("Shutting down room");
    let mut state = terracotta_state.lock().await;
    let room = state.try_shutdown_current().map_err(|e| {
        log::error!("Failed to shutdown room: {}", e);
        e.to_string()
    })?;
    *state = TerracottaState::Idle;
    Ok(room.code.clone())
}
