//! Modrinth / CurseForge 元数据请求命令，以及 CurseForge 可用性查询。
//!
//! 两个来源都只拼接官方地址，镜像与失败回退由下载源设置决定；
//! CurseForge 的 `x-api-key` 由 core 的服务来源解析附带。

use pcl_core::io::net::http::api_client::MinecraftApiClient;

#[tauri::command]
pub async fn fetch_with_modrinth(
    api_client: tauri::State<'_, &MinecraftApiClient>,
    endpoint: &str,
) -> Result<serde_json::Value, String> {
    api_client
        .fetch_with_modrinth(endpoint)
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn fetch_with_curseforge(
    api_client: tauri::State<'_, &MinecraftApiClient>,
    endpoint: &str,
) -> Result<serde_json::Value, String> {
    api_client
        .fetch_with_curseforge(endpoint)
        .await
        .map_err(|error| error.to_string())
}

/// 未配置密钥时界面需要隐藏 CurseForge 选项（对照 CE 的 SupportCurseForge）。
#[tauri::command]
pub fn curseforge_available() -> bool {
    crate::secrets::CURSEFORGE_API_KEY.is_some()
}
