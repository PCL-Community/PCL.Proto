use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::api_client::MinecraftApiClient;

#[derive(Serialize, Deserialize)]
struct UuidJson {
    id: String,
    name: String,
}

#[tauri::command]
pub async fn fetch_username_uuid(
    username: &str,
    client: tauri::State<'_, &MinecraftApiClient>,
) -> Result<String, String> {
    // ① 获取 UUID
    let profile_url = format!("https://api.mojang.com/users/profiles/minecraft/{username}");
    let uuid_data = client
        .get::<UuidJson>(&profile_url, true)
        .await
        .map_err(|err| err.to_string())?;
    let uuid = uuid_data.id;
    Ok(uuid)
}

#[tauri::command]
pub async fn fetch_uuid_profile(
    uuid: &str,
    client: tauri::State<'_, &MinecraftApiClient>,
) -> Result<Value, String> {
    let session_url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid}");
    let session_data = client
        .get::<Value>(&session_url, true)
        .await
        .map_err(|err| err.to_string());
    session_data
}
