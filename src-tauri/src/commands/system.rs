//! 系统级命令：外链、系统字体、今日人品和服务器列表 ping。

use chrono::Datelike;
use pcl_core::link::mcping::{self, model::McPingResult};
use pcl_core::utils::hash::get_pcl_hash;

#[tauri::command]
pub async fn open_external_url(url: String) -> Result<(), String> {
    crate::external::open_async(url).await
}

#[tauri::command]
pub async fn system_fonts() -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(|| {
        let mut db = fontdb::Database::new();
        db.load_system_fonts();
        let mut names: Vec<_> = db
            .faces()
            .flat_map(|face| face.families.iter().map(|(name, _)| name.clone()))
            .collect();
        names.sort();
        names.dedup();
        names
    })
    .await
    .map_err(|error| error.to_string())
}

#[tauri::command]
/// 获取今日人品，只考虑到日期
pub fn get_lucky_today() -> u8 {
    let identifier = &crate::config::manager::ConfigManager::instance().pcl_identifier;
    let now = chrono::Local::now();
    let str1 = format!("asdfgbn{}12#3$45{}IUY", now.day(), now.year());
    let hash1 = get_pcl_hash(&str1);
    let str2 = format!("QWERTY{}0*8&6{}kjhg", identifier, now.day());
    let hash2 = get_pcl_hash(&str2);
    let combined = (hash1 as f64 / 3.0 + hash2 as f64 / 3.0).abs();
    let num = ((combined / 527.0) % 1001.0).round() as i32;
    if num >= 970 {
        100
    } else {
        (num as f64 / 969.0 * 99.0).round() as u8
    }
}

#[tauri::command]
pub async fn server_query(addr_str: &str) -> Result<(McPingResult, u128), String> {
    mcping::server_query(addr_str).await
}
