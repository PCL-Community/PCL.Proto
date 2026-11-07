use crate::config::manager::ConfigManager;
use base64::{Engine, engine::general_purpose};
use pcl_core::io::net::http::api_client::MinecraftApiClient;
use pcl_core::minecraft::profile::skin;
use serde::Serialize;
use tauri::{AppHandle, Manager, path::BaseDirectory};
use tokio::{fs::File, io::AsyncWriteExt};

#[tauri::command]
pub async fn fetch_username_uuid(
    username: &str,
    client: tauri::State<'_, &MinecraftApiClient>,
) -> Result<String, String> {
    skin::username_uuid(&client, username).await
}

#[tauri::command]
pub async fn fetch_skin_url(
    uuid: &str,
    client: tauri::State<'_, &MinecraftApiClient>,
) -> Result<Option<String>, String> {
    let session = skin::session_profile(&client, uuid).await?;
    Ok(skin::texture_url(&session))
}

#[tauri::command]
pub async fn fetch_skin_from_url(
    app: AppHandle,
    api_client: tauri::State<'_, &MinecraftApiClient>,
    url: &str,
    uuid: &str,
) -> Result<String, String> {
    let skin_cache_dir = app.path().app_cache_dir().unwrap().join("skins");
    tokio::fs::create_dir_all(&skin_cache_dir)
        .await
        .map_err(|err| err.to_string())?;
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
    let skin_cache_path = app
        .path()
        .resolve(format!("skins/{uuid}.png"), BaseDirectory::AppCache)
        .map_err(|err| err.to_string())?;
    let skin_bytes = tokio::fs::read(&skin_cache_path)
        .await
        .map_err(|err| err.to_string())?;
    let base64_string = general_purpose::STANDARD.encode(&skin_bytes);
    Ok(format!("data:image/png;base64,{}", base64_string))
}

/// 档案皮肤，对照 CE 的 skinMs / skinAuth / skinLegacy 三个加载器。
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileSkin {
    /// 皮肤图片（data URL）；为空时界面使用 `default`。
    pub skin: Option<String>,
    /// 兜底的内置皮肤名，只会是 `Steve` 或 `Alex`。
    pub default: String,
}

/// 读取档案皮肤：正版走 Mojang 会话服务器，第三方走认证服务器自带的会话服务器，
/// 离线直接用内置皮肤。失败一律回退到内置皮肤（与 CE 一致）。
#[tauri::command]
pub async fn fetch_profile_skin(
    app: AppHandle,
    profile_id: String,
    refresh: bool,
) -> Result<ProfileSkin, String> {
    use crate::config::profiles as service;
    use pcl_core::minecraft::profile::ProfileType;

    let id = profile_id.clone();
    let profile = tauri::async_runtime::spawn_blocking(move || service::find(&id))
        .await
        .map_err(|error| error.to_string())?
        .ok_or("The source profile does not exist.")?;
    let default = skin::default_skin(&profile).to_owned();
    if profile.profile_type == ProfileType::Offline {
        return Ok(ProfileSkin {
            skin: profile
                .offline_skin
                .map(|skin| format!("data:image/png;base64,{}", skin.image)),
            default,
        });
    }
    // CE：离线 UUID 形式的账号在会话服务器上查不到皮肤。
    if skin::session_url(&profile).is_none() {
        return Ok(ProfileSkin {
            skin: None,
            default,
        });
    }
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("skins")
        .join(format!("{}.png", profile.uuid));
    if !refresh
        && let Ok(bytes) = tokio::fs::read(&cache).await
        && !bytes.is_empty()
    {
        return Ok(ProfileSkin {
            skin: Some(data_url(&bytes)),
            default,
        });
    }
    let client = ConfigManager::instance().api_client.http_client();
    let Some(bytes) = skin::fetch(&client, &profile).await? else {
        return Ok(ProfileSkin {
            skin: None,
            default,
        });
    };
    if let Some(parent) = cache.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    tokio::fs::write(&cache, &bytes)
        .await
        .map_err(|error| error.to_string())?;
    Ok(ProfileSkin {
        skin: Some(data_url(&bytes)),
        default,
    })
}

#[tauri::command]
pub async fn change_offline_skin(
    profile_id: String,
    source: Option<std::path::PathBuf>,
    slim: bool,
) -> Result<(), pcl_core::minecraft::profile::auth::AuthError> {
    use crate::config::profiles as service;
    use pcl_core::minecraft::profile::{OfflineSkin, ProfileType, auth::AuthError};

    let skin = if let Some(source) = source {
        let size = tokio::fs::metadata(&source)
            .await
            .map_err(|error| AuthError::new("skin_file", error.to_string()))?
            .len();
        if size > 24 * 1024 {
            return Err(AuthError::new("skin_too_large", "皮肤文件不能超过 24 KB"));
        }
        let bytes = tokio::fs::read(source)
            .await
            .map_err(|error| AuthError::new("skin_file", error.to_string()))?;
        skin::validate(&bytes)?;
        Some(OfflineSkin {
            image: general_purpose::STANDARD.encode(bytes),
            slim,
        })
    } else {
        None
    };
    service::blocking(move || {
        crate::config::manager::ConfigManager::instance()
            .update(|state| {
                let profile = state
                    .profile
                    .profiles
                    .iter_mut()
                    .find(|profile| profile.profile_id == profile_id)
                    .ok_or("The source profile does not exist.")?;
                if profile.profile_type != ProfileType::Offline {
                    return Err("Only offline profiles can use a local skin.".into());
                }
                profile.offline_skin = skin;
                Ok(())
            })
            .map_err(|error| AuthError::new("profile_store", error))
    })
    .await
}

fn data_url(bytes: &[u8]) -> String {
    use base64::Engine;
    format!(
        "data:image/png;base64,{}",
        base64::engine::general_purpose::STANDARD.encode(bytes)
    )
}

/// 皮肤缓存路径，保存与刷新皮肤都从这里取。
fn cache_path(app: &AppHandle, uuid: &str) -> Result<std::path::PathBuf, String> {
    Ok(app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?
        .join("skins")
        .join(format!("{uuid}.png")))
}

/// 把界面上正在显示的皮肤另存为用户选择的文件，对照 CE 的 `MySkin.Save`。
///
/// 图片由界面传入（自定义皮肤是后端给的 data URL，内置皮肤是打包资源），
/// 这样离线档案也能保存，且开发模式下同样可用。
#[tauri::command]
pub async fn save_profile_skin(
    destination: std::path::PathBuf,
    image: String,
) -> Result<(), String> {
    use base64::Engine;
    let encoded = image
        .split_once(",")
        .map_or(image.as_str(), |(_, data)| data)
        .trim();
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .map_err(|error| error.to_string())?;
    if bytes.is_empty() {
        return Err("皮肤还没有加载完成".into());
    }
    if let Some(parent) = destination.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|error| error.to_string())?;
    }
    tokio::fs::write(&destination, &bytes)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

/// 上传正版皮肤，对照 CE 的 `ProfileUi.ChangeSkinMs`。
///
/// 令牌过期时先静默刷新，仍然被拒绝再强制刷新重试一次。
#[tauri::command]
pub async fn change_microsoft_skin(
    app: AppHandle,
    profile_id: String,
    source: std::path::PathBuf,
    slim: bool,
) -> Result<(), pcl_core::minecraft::profile::auth::AuthError> {
    use crate::config::profiles;
    use crate::login;

    let id = profile_id.clone();
    let stored = tauri::async_runtime::spawn_blocking(move || profiles::find(&id))
        .await
        .map_err(|error| {
            pcl_core::minecraft::profile::auth::AuthError::new("profile_store", error.to_string())
        })?
        .ok_or_else(|| {
            pcl_core::minecraft::profile::auth::AuthError::new(
                "profile_not_found",
                "The source profile does not exist.",
            )
        })?;
    let bytes = tokio::fs::read(&source).await.map_err(|error| {
        pcl_core::minecraft::profile::auth::AuthError::new("skin_file", error.to_string())
    })?;
    skin::validate(&bytes)?;
    let file_name = source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "skin.png".into());

    let profile = login::login_for_launch(&stored, false).await?;
    let client = ConfigManager::instance().api_client.http_client();
    let uploaded =
        skin::upload_microsoft(&client, &profile.access_token, &bytes, &file_name, slim).await;
    match uploaded {
        // CE：令牌被拒绝时重新登录一次再上传。
        Err(error) if error.code == "invalid_token" => {
            let profile = login::refresh_microsoft(&stored).await?;
            skin::upload_microsoft(&client, &profile.access_token, &bytes, &file_name, slim)
                .await?;
        }
        result => result?,
    }
    // 换完皮肤立刻失效本地缓存，界面重新拉取即可看到新皮肤。
    let _ = tokio::fs::remove_file(cache_path(&app, &stored.uuid).unwrap_or_default()).await;
    Ok(())
}
