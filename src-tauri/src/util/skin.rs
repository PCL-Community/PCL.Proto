use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::core::api_client::{McApiResult, MinecraftApiClient};

#[derive(Serialize, Deserialize)]
struct UuidJson {
    id: String,
    name: String,
}

async fn fetch_username_uuid(username: &str, client: &MinecraftApiClient) -> McApiResult<String> {
    // ① 获取 UUID
    let profile_url = format!("https://api.mojang.com/users/profiles/minecraft/{username}");
    let uuid_data = client.get::<UuidJson>(&profile_url, true).await?;
    let uuid = uuid_data.id;
    Ok(uuid)
}

async fn fetch_uuid_profile(uuid: &str, client: &MinecraftApiClient) -> McApiResult<Value> {
    let session_url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid}");
    let session_data: Value = client.get(&session_url, true).await?;
    Ok(session_data)
}

#[tauri::command]
pub async fn fetch_username_profile(
    api_client: tauri::State<'_, &crate::core::api_client::MinecraftApiClient>,
    username: &str,
) -> Result<Value, String> {
    let uuid = fetch_username_uuid(username, &api_client)
        .await
        .map_err(|e| e.to_string())?;
    let session_data = fetch_uuid_profile(&uuid, &api_client)
        .await
        .map_err(|e| e.to_string());
    session_data
}
