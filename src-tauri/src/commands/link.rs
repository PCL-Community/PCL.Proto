//! Terracotta 联机桥接命令；具体实现在 `pcl-core` 的 `link` feature 里。

use pcl_core::link::scaffolding;

#[tauri::command]
pub fn get_terracotta_meta() -> serde_json::Value {
    scaffolding::get_terracotta_meta()
}

#[tauri::command]
pub fn get_terracotta_state() -> serde_json::Value {
    scaffolding::get_terracotta_state()
}

#[tauri::command]
pub fn set_terracotta_waiting() {
    scaffolding::set_terracotta_waiting()
}

#[tauri::command]
pub fn set_terracotta_host_scanning() {
    scaffolding::set_terracotta_host_scanning()
}

#[tauri::command]
pub fn set_terracotta_host_starting(mc_port: u16, player: String) -> Result<String, String> {
    scaffolding::set_terracotta_host_starting(mc_port, player)
}

#[tauri::command]
pub fn set_terracotta_guesting(room_code: String, player: String) -> Result<(), String> {
    scaffolding::set_terracotta_guesting(room_code, player)
}
