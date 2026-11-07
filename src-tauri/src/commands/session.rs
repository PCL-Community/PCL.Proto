//! 游戏会话类命令：查询运行中的进程、停止和清理日志。

use pcl_core::minecraft::launch::session::{self, GameSession};

#[tauri::command]
pub fn get_game_sessions(id: Option<String>) -> Vec<GameSession> {
    session::get_game_sessions(id)
}

#[tauri::command]
pub fn stop_game_session(id: String) -> Result<(), String> {
    session::stop_game_session(id)
}

#[tauri::command]
pub fn clear_game_session_log(id: String) -> Result<(), String> {
    session::clear_game_session_log(id)
}
