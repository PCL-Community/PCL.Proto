//! Minecraft 皮肤查询、会话协议、文件校验和上传；缓存与登录交互由宿主负责。

use base64::{Engine, engine::general_purpose};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{
    McProfile, ProfileType,
    auth::{AuthError, AuthResult},
    uuid,
};
use crate::io::net::http::api_client::MinecraftApiClient;

#[derive(Deserialize, Serialize)]
struct UuidJson {
    id: String,
}

pub async fn username_uuid(client: &MinecraftApiClient, username: &str) -> Result<String, String> {
    let profile_url = format!("https://api.mojang.com/users/profiles/minecraft/{username}");
    client
        .get::<UuidJson>(&profile_url, true)
        .await
        .map(|profile| profile.id)
        .map_err(|error| error.to_string())
}

pub async fn session_profile(client: &MinecraftApiClient, uuid: &str) -> Result<Value, String> {
    let session_url = format!("https://sessionserver.mojang.com/session/minecraft/profile/{uuid}");
    client
        .get::<Value>(&session_url, true)
        .await
        .map_err(|error| error.to_string())
}

pub async fn fetch(
    client: &reqwest::Client,
    profile: &McProfile,
) -> Result<Option<Vec<u8>>, String> {
    let Some(url) = session_url(profile) else {
        return Ok(None);
    };
    let session: Value = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .json()
        .await
        .map_err(|error| error.to_string())?;
    let Some(url) = texture_url(&session) else {
        return Ok(None);
    };
    let bytes = client
        .get(url)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .bytes()
        .await
        .map_err(|error| error.to_string())?;
    Ok(Some(bytes.to_vec()))
}

/// 会话服务器地址：正版用 Mojang，第三方用认证服务器自带的会话服务器。
pub fn session_url(profile: &McProfile) -> Option<String> {
    if profile.uuid.starts_with("00000") {
        return None;
    }
    let address = match profile.profile_type {
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
    }?;
    Some(format!("{address}{}", profile.uuid))
}

/// 从会话服务器响应里解出皮肤地址，对照 CE 的 `ModSkin.McSkinGetAddress`。
pub fn texture_url(session: &Value) -> Option<String> {
    let encoded = session
        .get("properties")?
        .as_array()?
        .iter()
        .find(|property| property.get("name").and_then(Value::as_str) == Some("textures"))?
        .get("value")?
        .as_str()?;
    let decoded = general_purpose::STANDARD.decode(encoded).ok()?;
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
pub fn default_skin(profile: &McProfile) -> &'static str {
    match profile.profile_type {
        // 离线档案按自己的 UUID 选择默认皮肤。
        ProfileType::Offline => uuid::default_skin(&profile.uuid),
        // CE 的正版兜底按名称重新算一个离线 UUID。
        ProfileType::Microsoft => uuid::default_skin(&uuid::offline(&profile.user_name)),
        // 第三方兜底固定 Steve。
        _ => "Steve",
    }
}

/// 皮肤文件校验，对照 CE 的 `ModSkin.McSkinSelect`：64×32 或 64×64，且不超过 24 KB。
pub fn validate(bytes: &[u8]) -> AuthResult<()> {
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

#[cfg(feature = "microsoft")]
pub async fn upload_microsoft(
    client: &reqwest::Client,
    access_token: &str,
    bytes: &[u8],
    file_name: &str,
    slim: bool,
) -> AuthResult<()> {
    validate(bytes)?;
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
    use super::{png_size, validate};

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
        assert!(validate(&header(64, 64)).is_ok());
        assert!(validate(&header(64, 32)).is_ok());
        assert_eq!(
            validate(&header(32, 32)).unwrap_err().code,
            "skin_invalid_size"
        );
        let mut oversized = header(64, 64);
        oversized.extend_from_slice(&vec![0u8; 25 * 1024]);
        assert_eq!(validate(&oversized).unwrap_err().code, "skin_too_large");
    }
}
