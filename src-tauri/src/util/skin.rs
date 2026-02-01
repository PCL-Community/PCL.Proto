use crate::core::api_client::MinecraftApiClient;
use base64::{Engine, engine::general_purpose};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager, path::BaseDirectory};
use tokio::{fs::File, io::AsyncWriteExt};

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

#[tauri::command]
pub async fn fetch_skin_from_url(
    app: AppHandle,
    api_client: tauri::State<'_, &MinecraftApiClient>,
    url: &str,
    uuid: &str,
) -> Result<String, String> {
    let skin_cache_dir = app.path().app_cache_dir().unwrap().join("skins");
    std::fs::create_dir_all(&skin_cache_dir).map_err(|err| err.to_string())?;
    let skin_bytes = api_client
        .get_bytes(url)
        .await
        .map_err(|err| err.to_string())?;
    let skin_path = skin_cache_dir.join(format!("{uuid}.png"));
    let mut file = File::create(&skin_path)
        .await
        .map_err(|err| err.to_string())?;
    file.write_all(&skin_bytes)
        .await
        .map_err(|err| err.to_string())?;
    let base64_string = general_purpose::STANDARD.encode(&skin_bytes);
    Ok(format!("data:image/png;base64,{}", base64_string))
}

#[tauri::command]
pub async fn fetch_skin_from_uuid_cached(app: AppHandle, uuid: &str) -> Result<String, String> {
    let skin_cache_path = app.path().resolve(format!("skins/{uuid}.png"), BaseDirectory::AppCache).map_err(|err| err.to_string())?;
    let skin_bytes = tokio::fs::read(&skin_cache_path).await.map_err(|err| err.to_string())?;
    let base64_string = general_purpose::STANDARD.encode(&skin_bytes);
    Ok(format!("data:image/png;base64,{}", base64_string))
}
