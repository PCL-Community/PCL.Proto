//! Referenced from PCL-Community/PCL-CE
use crate::setup::ConfigManager;

use super::get_pcl_hash;
use chrono::{Datelike, Local};

#[tauri::command]
/// 获取今日人品，只考虑到日期
pub fn get_lucky_today() -> u8 {
    let pcl_identifier = &ConfigManager::instance().pcl_identifier;
    let now = Local::now();
    let str1 = format!("asdfgbn{}12#3$45{}IUY", now.day(), now.year());
    let hash1 = get_pcl_hash(&str1);
    let str2 = format!("QWERTY{}0*8&6{}kjhg", pcl_identifier, now.day());
    let hash2 = get_pcl_hash(&str2);
    let combined = (hash1 as f64 / 3.0 + hash2 as f64 / 3.0).abs();
    let num = ((combined / 527.0) % 1001.0).round() as i32;
    if num >= 970 {
        100
    } else {
        (num as f64 / 969.0 * 99.0).round() as u8
    }
}
