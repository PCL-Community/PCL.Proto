use super::terracotta::room::RoomCode;
use crate::scaffolding::terracotta::room::PUBLIC_SERVERS;
use crate::scaffolding::terracotta::states::TerracottaState;
use std::sync::Arc;
use tokio::sync::Mutex;
use tauri::State;

/// 启动联机中心
#[tauri::command]
pub async fn start_host(
    terracotta_state: State<'_, Arc<Mutex<TerracottaState>>>,
    player_name: &str,
    port: u16,
) -> Result<String, String> {
    log::info!("Starting host with player: {}, port: {}", player_name, port);
    let room_code = RoomCode::generate();
    log::info!("Generated room code: {}", room_code.code);
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::HostStarting {
            room: room_code.clone(),
            port,
        };
        drop(state);
    }
    log::info!("Attempting to start room host with public servers: {:?}", PUBLIC_SERVERS);
    let network_instance = room_code
        .start_room_host(port, Some(player_name), PUBLIC_SERVERS)
        .map_err(|e| {
            log::error!("Failed to start room host: {}", e);
            e.to_string()
        })?;
    log::info!("Room host started successfully with code: {}", room_code.code);
    Ok(room_code.code)
}

/// 加入联机
#[tauri::command]
pub async fn start_guest(
    terracotta_state: State<'_, Arc<tokio::sync::Mutex<TerracottaState>>>,
    room_code: &str,
    player_name: &str,
) -> Result<(), String> {
    log::info!("Starting guest with room code: {}, player: {}", room_code, player_name);
    // 解析房间码
    let room_code = RoomCode::from_code(room_code).ok_or_else(|| {
        log::error!("Invalid room code provided: {}", room_code);
        "Invalid room code".to_string()
    })?;
    log::info!("Successfully parsed room code: {}", room_code.code);
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::GuestConnecting {
            room: room_code.clone(),
        };
        drop(state);
    }
    log::info!("Attempting to join room as guest with public servers: {:?}", PUBLIC_SERVERS);
    room_code.start_room_guest(Some(player_name), PUBLIC_SERVERS);
    log::info!("Guest connection initiated successfully");
    Ok(())
}
