use super::terracotta::room::RoomCode;
// use super::terracotta::room::RoomInfo;
use super::terracotta::states::ScaffoldingGlobalState;
use tauri::State;

/// 生成房间码
#[tauri::command]
pub fn generate_room_code() -> Result<RoomCode, String> {
    let room_code = RoomCode::generate();
    Ok(room_code)
}

/// 解析房间码
#[tauri::command]
pub fn parse_room_code(code: &str) -> Result<RoomCode, String> {
    if let Some(room_code) = RoomCode::from_code(code) {
        Ok(room_code)
    } else {
        Err("Invalid room code".to_string())
    }
}

/// 启动联机中心
#[tauri::command]
pub async fn start_host(
    global_state: State<'_, ScaffoldingGlobalState>,
    room_code: RoomCode,
    player_name: &str,
) -> Result<(), String> {
    // // 创建房间状态
    // let room_state = RoomState::new(
    //     &room_code.code,
    //     &room_code.network_name,
    //     &room_code.network_secret,
    //     port,
    // );

    // // 保存房间状态
    // global_state.set_room_state(room_state).await;

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
    // let room_info = parse_room_code(room_code)?;

    // // 生成EasyTier网络配置
    // let mut easy_tier_manager = EasyTierManager::new();
    // let _config = easy_tier_manager
    //     .generate_network_config(
    //         &room_info.network_name,
    //         &room_info.network_secret,
    //         None, // 客不需要设置Hostname
    //     )
    //     .map_err(|e| e.to_string())?;

    // // 创建房间状态
    // let room_state = RoomState::new(
    //     &room_info.code,
    //     &room_info.network_name,
    //     &room_info.network_secret,
    //     port,
    // );

    // 保存房间状态
    // global_state.set_room_state(room_state).await;

    Ok(())
}

// /// 获取房间状态
// #[tauri::command]
// pub async fn get_room_state(
//     global_state: State<'_, ScaffoldingGlobalState>,
// ) -> Result<Option<RoomState>, String> {
//     Ok(global_state.get_room_state().await)
// }

/// 离开联机
#[tauri::command]
pub async fn leave_room(global_state: State<'_, ScaffoldingGlobalState>) -> Result<(), String> {
    // global_state.clear_room_state().await;
    Ok(())
}
