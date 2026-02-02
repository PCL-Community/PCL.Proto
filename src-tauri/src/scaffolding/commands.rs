use super::PlayerInfo;
use super::RoomInfo;
use super::easytier::EasyTierManager;
use super::terracotta::room::RoomCode;
use super::terracotta::states::{RoomState, ScaffoldingGlobalState};
use tauri::State;

/// 生成房间码
#[tauri::command]
pub fn generate_room_code() -> Result<RoomInfo, String> {
    let room_code = RoomCode::generate();
    
    Ok(RoomInfo {
        code: room_code.code,
        network_name: room_code.network_name,
        network_secret: room_code.network_secret,
        server_port: 25565, // 默认Minecraft服务器端口
    })
}

/// 解析房间码
#[tauri::command]
pub fn parse_room_code(code: &str) -> Result<RoomInfo, String> {
    if let Some(room_code) = RoomCode::parse(code) {
        Ok(RoomInfo {
            code: room_code.code,
            network_name: room_code.network_name,
            network_secret: room_code.network_secret,
            server_port: 25565, // 默认Minecraft服务器端口
        })
    } else {
        Err("Invalid room code".to_string())
    }
}

/// 启动联机中心
#[tauri::command]
pub async fn start_host(
    global_state: State<'_, ScaffoldingGlobalState>,
    room_info: RoomInfo,
    player_name: &str,
) -> Result<(), String> {
    // 生成Hostname
    let hostname = EasyTierManager::generate_hostname(room_info.server_port);
    
    // 生成EasyTier网络配置
    let mut easy_tier_manager = EasyTierManager::new();
    let _config = easy_tier_manager
        .generate_network_config(
            &room_info.network_name,
            &room_info.network_secret,
            Some(&hostname),
        )
        .map_err(|e| e.to_string())?;
    
    // 创建房间状态
    let room_state = RoomState::new(
        &room_info.code,
        &room_info.network_name,
        &room_info.network_secret,
        room_info.server_port,
    );
    
    // 保存房间状态
    global_state.set_room_state(room_state).await;
    
    Ok(())
}

/// 加入联机
#[tauri::command]
pub async fn join_room(
    global_state: State<'_, ScaffoldingGlobalState>,
    room_code: &str,
    player_name: &str,
) -> Result<(), String> {
    // 解析房间码
    let room_info = parse_room_code(room_code)?;
    
    // 生成EasyTier网络配置
    let mut easy_tier_manager = EasyTierManager::new();
    let _config = easy_tier_manager
        .generate_network_config(
            &room_info.network_name,
            &room_info.network_secret,
            None, // 客不需要设置Hostname
        )
        .map_err(|e| e.to_string())?;
    
    // 创建房间状态
    let room_state = RoomState::new(
        &room_info.code,
        &room_info.network_name,
        &room_info.network_secret,
        room_info.server_port,
    );
    
    // 保存房间状态
    global_state.set_room_state(room_state).await;
    
    Ok(())
}

/// 获取房间状态
#[tauri::command]
pub async fn get_room_state(
    global_state: State<'_, ScaffoldingGlobalState>,
) -> Result<Option<RoomState>, String> {
    Ok(global_state.get_room_state().await)
}

/// 离开联机
#[tauri::command]
pub async fn leave_room(
    global_state: State<'_, ScaffoldingGlobalState>,
) -> Result<(), String> {
    global_state.clear_room_state().await;
    Ok(())
}
