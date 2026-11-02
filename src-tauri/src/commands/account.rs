//! 账号命令：档案管理、登录会话与第三方认证服务器探测。
//!
//! 档案读写会阻塞式锁住配置，所以统一放进 `spawn_blocking`。

use serde::{Deserialize, Serialize};

use crate::minecraft::profile::auth::session::{self, LoginRequest, View};
use crate::minecraft::profile::auth::{AuthError, AuthResult, authlib, connect, microsoft};
use crate::minecraft::profile::{McProfile, ProfileType, SafeProfile, service, uuid};

/// 档案列表与当前选择，界面一次拿全。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProfileList {
    pub profiles: Vec<SafeProfile>,
    pub current: Option<String>,
}

impl ProfileList {
    fn load() -> AuthResult<Self> {
        let (profiles, current) = service::snapshot();
        Ok(Self {
            profiles: profiles.iter().map(SafeProfile::from).collect(),
            current,
        })
    }
}

/// 离线 UUID 的生成方式，对照 CE 的三个单选项。
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum UuidMode {
    Standard,
    Legacy,
    Custom,
}

impl UuidMode {
    fn resolve(self, user_name: &str, custom: Option<&str>) -> AuthResult<String> {
        match self {
            UuidMode::Standard => Ok(uuid::offline(user_name)),
            UuidMode::Legacy => Ok(uuid::legacy(user_name)),
            UuidMode::Custom => {
                uuid::normalize_custom(custom.unwrap_or_default()).ok_or_else(|| {
                    AuthError::rejected(
                        Some("invalid_uuid".into()),
                        Some("UUID 必须是 32 位十六进制字符。".into()),
                    )
                })
            }
        }
    }
}

/// 第三方认证服务器的探测结果，对照 CE 的 `PageLoginAuth.IsOAuthSupportedAsync`。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthServerInfo {
    /// 跟随 `X-Authlib-Injector-Api-Location` 之后的 API 根地址。
    pub api_location: String,
    pub server_name: Option<String>,
    /// 是否支持 Yggdrasil Connect（OAuth）登录。
    pub oauth_supported: bool,
    pub discovery_address: Option<String>,
    pub client_id: Option<String>,
    /// 服务器主页，用于“访问官网”。
    pub website: Option<String>,
}

#[tauri::command]
pub async fn get_profiles() -> Result<ProfileList, AuthError> {
    store(ProfileList::load).await
}

/// 选择档案；传 `null` 表示取消选择（CE 的“选择档案”按钮会清空当前档案）。
#[tauri::command]
pub async fn select_profile(profile_id: Option<String>) -> Result<ProfileList, AuthError> {
    store(move || {
        service::select(profile_id.as_deref())?;
        ProfileList::load()
    })
    .await
}

#[tauri::command]
pub async fn delete_profile(profile_id: String) -> Result<ProfileList, AuthError> {
    store(move || {
        service::remove(&profile_id)?;
        ProfileList::load()
    })
    .await
}

#[tauri::command]
pub async fn create_offline_profile(
    user_name: String,
    uuid_mode: UuidMode,
    custom_uuid: Option<String>,
) -> Result<ProfileList, AuthError> {
    let name = user_name.trim().to_owned();
    if name.is_empty() {
        return Err(AuthError::rejected(
            Some("invalid_player_id".into()),
            Some("玩家 ID 不能为空。".into()),
        ));
    }
    let uuid = uuid_mode.resolve(&name, custom_uuid.as_deref())?;
    store(move || {
        let mut profile = McProfile::offline(&name, &uuid);
        // CE 用随机 GUID 作为档案身份，UUID 相同也允许重复添加同名离线档案。
        profile.profile_id = service::new_profile_id();
        // 对照 CE 的 McLoginLegacyStart：离线档案的令牌就是 UUID。
        profile.access_token = uuid.clone();
        profile.client_token = uuid;
        service::add(&mut profile, true)?;
        ProfileList::load()
    })
    .await
}

#[tauri::command]
pub async fn set_offline_uuid(
    profile_id: String,
    uuid_mode: UuidMode,
    custom_uuid: Option<String>,
) -> Result<ProfileList, AuthError> {
    let lookup = profile_id.clone();
    let profile = store(move || {
        service::find(&lookup).ok_or_else(|| {
            AuthError::new("profile_not_found", "The source profile does not exist.")
        })
    })
    .await?;
    let value = uuid_mode.resolve(&profile.user_name, custom_uuid.as_deref())?;
    store(move || {
        let mut updated = profile.clone();
        updated.uuid = value.clone();
        updated.access_token = value;
        service::replace(&profile_id, updated)?;
        service::select_profile(&profile_id)?;
        ProfileList::load()
    })
    .await
}

/// 修改正版玩家 ID，对照 CE 的 `ProfileUi.EditProfileId` 微软分支。
#[tauri::command]
pub async fn rename_microsoft_profile(
    profile_id: String,
    user_name: String,
) -> Result<ProfileList, AuthError> {
    let name = user_name.trim();
    if !(3..=16).contains(&name.chars().count())
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(AuthError::rejected(
            Some("invalid_player_id".into()),
            Some("玩家 ID 只能是 3 到 16 位的字母、数字或下划线。".into()),
        ));
    }
    let lookup = profile_id.clone();
    let profile = store(move || {
        service::find(&lookup).ok_or_else(|| {
            AuthError::new("profile_not_found", "The source profile does not exist.")
        })
    })
    .await?;
    if profile.profile_type != ProfileType::Microsoft {
        return Err(AuthError::configuration("只有正版账号能在启动器里改名"));
    }
    let provider = microsoft::MicrosoftProvider::new()?;
    let renamed = provider.change_name(&profile.access_token, name).await?;
    store(move || {
        let mut updated = profile.clone();
        updated.user_name = renamed;
        service::replace(&profile_id, updated)?;
        service::select_profile(&profile_id)?;
        ProfileList::load()
    })
    .await
}

/// 探测第三方认证服务器：解析 API 地址并判断是否支持 OAuth 登录。
#[tauri::command]
pub async fn probe_auth_server(address: String) -> Result<AuthServerInfo, AuthError> {
    let address = address.trim().to_owned();
    let client = crate::minecraft::profile::auth::http_client()?;
    let api_location = authlib::resolve_api_location(&client, &address).await;
    let normalized = authlib::normalize_server(&api_location)?;
    // CE：主机名命中内置 ClientId 时直接认为支持 OAuth。
    let built_in = reqwest::Url::parse(&normalized).ok().and_then(|url| {
        url.host_str()
            .and_then(connect::built_in_client_id)
            .map(str::to_owned)
    });
    let server_name = authlib::server_name(&client, &normalized).await;
    let discovery = fetch_discovery(&client, &normalized).await;
    // 发现文档能通过 CE 的全部校验（HTTPS、issuer、必须 scope、设备端点、ClientId）才算支持。
    let resolved = match &discovery {
        Some(address) => connect::initialize(&client, address, built_in.as_deref())
            .await
            .ok(),
        None => None,
    };
    Ok(AuthServerInfo {
        website: reqwest::Url::parse(&normalized)
            .ok()
            .map(|url| format!("{}://{}/", url.scheme(), url.host_str().unwrap_or_default())),
        api_location: normalized,
        server_name,
        oauth_supported: built_in.is_some() || resolved.is_some(),
        discovery_address: resolved
            .as_ref()
            .map(|metadata| metadata.address.clone())
            .or(discovery),
        client_id: resolved
            .as_ref()
            .map(|metadata| metadata.client_id.clone())
            .or(built_in),
    })
}

/// 从 authlib-injector 元数据里取出 OpenID 发现地址。
async fn fetch_discovery(client: &reqwest::Client, api_root: &str) -> Option<String> {
    let response = client.get(api_root).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let metadata: serde_json::Value = response.json().await.ok()?;
    connect::discovery_address_of(&metadata)
}

#[tauri::command]
pub async fn start_login(request: LoginRequest) -> Result<String, AuthError> {
    session::start(request)
}

#[tauri::command]
pub async fn login_state(session_id: String) -> Result<View, AuthError> {
    session::view(&session_id)
        .ok_or_else(|| AuthError::new("session_not_found", "登录会话已结束，请重新登录。"))
}

#[tauri::command]
pub async fn cancel_login(session_id: String) {
    session::cancel(&session_id);
}

#[tauri::command]
pub async fn select_login_profile(
    session_id: String,
    candidate_id: Option<String>,
) -> Result<(), AuthError> {
    session::select(&session_id, candidate_id)
}

/// 档案读写离开异步上下文执行。
async fn store<T: Send + 'static>(
    run: impl FnOnce() -> AuthResult<T> + Send + 'static,
) -> AuthResult<T> {
    tauri::async_runtime::spawn_blocking(run)
        .await
        .map_err(|error| AuthError::new("profile_store", error.to_string()))?
}
