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
    let room_code = RoomCode::generate();
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::HostStarting {
            room: room_code.clone(),
            port,
        };
        drop(state);
    }
    // let network_instance = room_code
    //     .start_room_host(port, Some(player_name), PUBLIC_SERVERS)
    //     .map_err(|e| e.to_string())?;
    Ok(room_code.code)
}

/// 加入联机
#[tauri::command]
pub async fn start_guest(
    terracotta_state: State<'_, Arc<tokio::sync::Mutex<TerracottaState>>>,
    room_code: &str,
    player_name: &str,
) -> Result<(), String> {
    // 解析房间码
    let room_code = RoomCode::from_code(room_code).ok_or("Invalid room code".to_string())?;
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::GuestConnecting {
            room: room_code.clone(),
        };
        drop(state);
    }
    room_code.start_room_guest(Some(player_name), PUBLIC_SERVERS);
    Ok(())
}
