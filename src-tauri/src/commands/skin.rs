use crate::io::net::http::api_client::MinecraftApiClient;
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
    use crate::minecraft::profile::{ProfileType, service};

    let id = profile_id.clone();
    let profile = tauri::async_runtime::spawn_blocking(move || service::find(&id))
        .await
        .map_err(|error| error.to_string())?
        .ok_or("The source profile does not exist.")?;
    let default = default_skin(&profile);
    if profile.profile_type == ProfileType::Offline {
        return Ok(ProfileSkin {
            skin: None,
            default,
        });
    }
    // CE：离线 UUID 形式的账号在会话服务器上查不到皮肤。
    if profile.uuid.starts_with("00000") {
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
    let client =
        crate::minecraft::profile::auth::http_client().map_err(|error| error.to_string())?;
    let Some(address) = skin_address(&profile) else {
        return Ok(ProfileSkin {
            skin: None,
            default,
        });
    };
    let profile_url = format!("{address}{}", profile.uuid);
    let session: Value = client
        .get(&profile_url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json()
        .await
        .map_err(|error| error.to_string())?;
    let Some(skin_url) = texture_url(&session) else {
        // 用户没有自定义皮肤时 CE 直接用内置皮肤。
        return Ok(ProfileSkin {
            skin: None,
            default,
        });
    };
    let bytes = client
        .get(&skin_url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .bytes()
        .await
        .map_err(|error| error.to_string())?;
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

/// 会话服务器地址：正版用 Mojang，第三方用认证服务器自带的会话服务器。
fn skin_address(profile: &crate::minecraft::profile::McProfile) -> Option<String> {
    use crate::minecraft::profile::ProfileType;
    match profile.profile_type {
        ProfileType::Microsoft => {
            Some("https://sessionserver.mojang.com/session/minecraft/profile/".to_owned())
        }
        ProfileType::Authlib | ProfileType::YggdrasilConnect => {
            profile.server.as_deref().map(|server| {
                format!(
                    "{}/sessionserver/session/minecraft/profile/",
                    server.replace("/authserver", "").trim_end_matches('/')
                )
            })
        }
        ProfileType::Offline => None,
    }
}

/// 从会话服务器响应里解出皮肤地址，对照 CE 的 `ModSkin.McSkinGetAddress`。
fn texture_url(session: &Value) -> Option<String> {
    let encoded = session
        .get("properties")?
        .as_array()?
        .iter()
        .find(|property| property.get("name").and_then(Value::as_str) == Some("textures"))?
        .get("value")?
        .as_str()?;
    use base64::Engine;
    let decoded = base64::engine::general_purpose::STANDARD
        .decode(encoded)
        .ok()?;
    // CE 会把整段 JSON 转小写后再取 textures.skin.url。
    let textures: Value =
        serde_json::from_str(&String::from_utf8_lossy(&decoded).to_lowercase()).ok()?;
    textures
        .pointer("/textures/skin/url")
        .and_then(Value::as_str)
        .map(|url| {
            if url.contains("minecraft.net/") {
                url.replacen("http://", "https://", 1)
            } else {
                url.to_owned()
            }
        })
}

/// 没有自定义皮肤时按 UUID 选择内置皮肤，对照 CE 的 `McSkinSex`。
fn default_skin(profile: &crate::minecraft::profile::McProfile) -> String {
    use crate::minecraft::profile::{ProfileType, uuid};
    match profile.profile_type {
        // 离线档案按自己的 UUID 选择默认皮肤。
        ProfileType::Offline => uuid::default_skin(&profile.uuid).to_owned(),
        // CE 的正版兜底按名称重新算一个离线 UUID。
        ProfileType::Microsoft => uuid::default_skin(&uuid::offline(&profile.user_name)).to_owned(),
        // 第三方兜底固定 Steve。
        _ => "Steve".to_owned(),
    }
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
) -> Result<(), crate::minecraft::profile::auth::AuthError> {
    use crate::minecraft::profile::auth::session;
    use crate::minecraft::profile::service;

    let id = profile_id.clone();
    let stored = tauri::async_runtime::spawn_blocking(move || service::find(&id))
        .await
        .map_err(|error| {
            crate::minecraft::profile::auth::AuthError::new("profile_store", error.to_string())
        })?
        .ok_or_else(|| {
            crate::minecraft::profile::auth::AuthError::new(
                "profile_not_found",
                "The source profile does not exist.",
            )
        })?;
    let bytes = tokio::fs::read(&source).await.map_err(|error| {
        crate::minecraft::profile::auth::AuthError::new("skin_file", error.to_string())
    })?;
    validate_skin(&bytes)?;
    let file_name = source
        .file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_else(|| "skin.png".into());

    let profile = session::login_for_launch(&stored, false).await?;
    let uploaded = upload_skin(&profile.access_token, &bytes, &file_name, slim).await;
    match uploaded {
        // CE：令牌被拒绝时重新登录一次再上传。
        Err(error) if error.code == "invalid_token" => {
            let profile = session::refresh_microsoft(&stored).await?;
            upload_skin(&profile.access_token, &bytes, &file_name, slim).await?;
        }
        result => result?,
    }
    // 换完皮肤立刻失效本地缓存，界面重新拉取即可看到新皮肤。
    let _ = tokio::fs::remove_file(cache_path(&app, &stored.uuid).unwrap_or_default()).await;
    Ok(())
}

/// 皮肤文件校验，对照 CE 的 `ModSkin.McSkinSelect`：64×32 或 64×64，且不超过 24 KB。
fn validate_skin(bytes: &[u8]) -> Result<(), crate::minecraft::profile::auth::AuthError> {
    use crate::minecraft::profile::auth::AuthError;
    if bytes.len() > 24 * 1024 {
        return Err(AuthError::new(
            "skin_too_large",
            format!(
                "皮肤文件过大（{:.2} KB），上限为 24 KB",
                bytes.len() as f64 / 1024.0
            ),
        ));
    }
    let Some((width, height)) = png_size(bytes) else {
        return Err(AuthError::new("skin_invalid", "无法读取皮肤图片"));
    };
    if width != 64 || !matches!(height, 32 | 64) {
        return Err(AuthError::new(
            "skin_invalid_size",
            format!("皮肤尺寸必须是 64×32 或 64×64，当前为 {width}×{height}"),
        ));
    }
    Ok(())
}

/// 从 PNG 的 IHDR 里读尺寸，避免为一个校验引入图像库。
fn png_size(bytes: &[u8]) -> Option<(u32, u32)> {
    let header = bytes.get(..24)?;
    if header.starts_with(&[0x89, b'P', b'N', b'G']) && &header[12..16] == b"IHDR" {
        return Some((
            u32::from_be_bytes(header[16..20].try_into().ok()?),
            u32::from_be_bytes(header[20..24].try_into().ok()?),
        ));
    }
    None
}

async fn upload_skin(
    access_token: &str,
    bytes: &[u8],
    file_name: &str,
    slim: bool,
) -> Result<(), crate::minecraft::profile::auth::AuthError> {
    use crate::minecraft::profile::auth::AuthError;
    let client = crate::minecraft::profile::auth::http_client()?;
    let form = reqwest::multipart::Form::new()
        .text("variant", if slim { "slim" } else { "classic" })
        .part(
            "file",
            reqwest::multipart::Part::bytes(bytes.to_vec()).file_name(file_name.to_owned()),
        );
    let response = client
        .post("https://api.minecraftservices.com/minecraft/profile/skins")
        .bearer_auth(access_token)
        .multipart(form)
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    if status == reqwest::StatusCode::UNAUTHORIZED
        || text.contains("request requires user authentication")
    {
        return Err(AuthError::new(
            "invalid_token",
            "正版登录状态已失效，请重新登录后再试。",
        ));
    }
    if !status.is_success() {
        return Err(AuthError::network(format!(
            "皮肤上传返回 HTTP {status}：{}",
            text.chars().take(200).collect::<String>()
        )));
    }
    Ok(())
}

#[cfg(test)]
mod skin_tests {
    use super::{png_size, validate_skin};

    /// 一个最小的 64×64 PNG 头。
    fn header(width: u32, height: u32) -> Vec<u8> {
        let mut bytes = vec![0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];
        bytes.extend_from_slice(&[0, 0, 0, 13]);
        bytes.extend_from_slice(b"IHDR");
        bytes.extend_from_slice(&width.to_be_bytes());
        bytes.extend_from_slice(&height.to_be_bytes());
        bytes
    }

    #[test]
    fn png_sizes_are_read_from_ihdr() {
        assert_eq!(png_size(&header(64, 64)), Some((64, 64)));
        assert_eq!(png_size(&header(64, 32)), Some((64, 32)));
        assert_eq!(png_size(b"not a png at all!!!!!"), None);
    }

    #[test]
    fn skins_must_be_64_wide_and_small() {
        assert!(validate_skin(&header(64, 64)).is_ok());
        assert!(validate_skin(&header(64, 32)).is_ok());
        assert_eq!(
            validate_skin(&header(32, 32)).unwrap_err().code,
            "skin_invalid_size"
        );
        let mut oversized = header(64, 64);
        oversized.extend_from_slice(&vec![0u8; 25 * 1024]);
        assert_eq!(
            validate_skin(&oversized).unwrap_err().code,
            "skin_too_large"
        );
    }
}
