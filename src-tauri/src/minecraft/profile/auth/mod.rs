//! Authentication providers, mirroring `PCL.Core.Minecraft.Profile.Authentication`.
//!
//! CE hides the Microsoft six-step protocol and the two Yggdrasil flavours behind
//! `IAuthenticateProvider`. Proto keeps the same provider split, but the interactive
//! parts (device-code dialog, profile picker) live in [`session`] rather than in WPF
//! dialogs, so a provider is a plain set of async functions.

pub mod authlib;
pub mod connect;
pub mod microsoft;
pub mod oauth;
pub mod session;
pub mod yggdrasil;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::minecraft::profile::{McProfile, ProfileType};

pub type AuthResult<T> = Result<T, AuthError>;

/// Authentication failure carrying a stable machine-readable code.
///
/// Mirrors `IdentityModelException`: the UI maps `code` (`xsts_2148916233`,
/// `minecraft_not_owned`, `profile_not_created`, `invalid_grant`, ...) onto a
/// localized dialog, while `message` keeps the detail for logs.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthError {
    pub code: String,
    pub message: String,
}

impl AuthError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// 认证服务器返回的错误（CE 的 `IdentityModelAuthenticationException`）。
    pub fn rejected(code: Option<String>, message: Option<String>) -> Self {
        Self::new(
            code.unwrap_or_else(|| "invalid_response".into()),
            message.unwrap_or_else(|| "The authentication server rejected the request.".into()),
        )
    }

    /// 配置缺失或非法（CE 的 `IdentityModelConfigurationException`）。
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::new("configuration_error", message)
    }

    /// 网络或响应解析失败。
    pub fn network(message: impl Into<String>) -> Self {
        Self::new("network_error", message)
    }

    /// 刷新令牌已失效：唯一一种可以直接退回完整登录的刷新失败（CE 的 `invalid_grant`）。
    pub fn is_invalid_grant(&self) -> bool {
        self.code.eq_ignore_ascii_case("invalid_grant")
    }

    /// XSTS 账号级错误需要专门的用户指引，不能靠重新登录恢复。
    pub fn is_xsts(&self) -> bool {
        self.code.to_ascii_lowercase().starts_with("xsts_")
    }
}

impl std::fmt::Display for AuthError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}: {}", self.code, self.message)
    }
}

impl std::error::Error for AuthError {}

impl From<reqwest::Error> for AuthError {
    fn from(error: reqwest::Error) -> Self {
        Self::network(error.to_string())
    }
}

/// One selectable player profile offered by an authentication server.
///
/// Mirrors `AuthenticationCandidate`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Candidate {
    pub id: String,
    pub name: String,
}

/// Successful authentication, mirroring `AuthenticationResult`.
#[derive(Clone, Debug)]
pub struct AuthenticationResult {
    pub profile_type: ProfileType,
    pub user_name: String,
    pub uuid: String,
    pub access_token: String,
    pub refresh_token: String,
    pub client_token: String,
    pub token_type: String,
    pub expires_at: Option<DateTime<Utc>>,
    pub raw_json: String,
    pub server: Option<String>,
    pub server_name: Option<String>,
    pub login_name: Option<String>,
    pub password: Option<String>,
    pub provider: Option<String>,
    pub discovery_address: Option<String>,
    pub id_token: Option<String>,
    /// Proto-only: `${auth_xuid}` launch argument for Microsoft profiles.
    pub xuid: Option<String>,
}

impl AuthenticationResult {
    /// CE 的 `ProfileService.ApplyAuthenticationResult`：写回档案字段，
    /// `None` 表示服务器没有提供该字段，保留档案原有值。
    pub fn apply_to(&self, profile: &mut McProfile) {
        profile.profile_type = self.profile_type;
        profile.user_name = self.user_name.clone();
        profile.uuid = self.uuid.clone();
        profile.access_token = self.access_token.clone();
        profile.refresh_token = self.refresh_token.clone();
        profile.client_token = self.client_token.clone();
        profile.token_type = self.token_type.clone();
        profile.expires_at = self.expires_at;
        profile.raw_json = self.raw_json.clone();
        if self.server.is_some() {
            profile.server = self.server.clone();
        }
        if self.server_name.is_some() {
            profile.server_name = self.server_name.clone();
        }
        if self.login_name.is_some() {
            profile.login_name = self.login_name.clone();
        }
        if self.password.is_some() {
            profile.password = self.password.clone();
        }
        if self.provider.is_some() {
            profile.provider = self.provider.clone();
        }
        if self.discovery_address.is_some() {
            profile.discovery_address = self.discovery_address.clone();
        }
        if self.id_token.is_some() {
            profile.id_token = self.id_token.clone();
        }
        if self.xuid.is_some() {
            profile.xuid = self.xuid.clone();
        }
    }
}

/// HTTP client that honours the launcher proxy policy (CE 的 `NetworkService.GetClient`)。
pub fn http_client() -> AuthResult<reqwest::Client> {
    crate::io::net::proxy::builder()
        .build()
        .map_err(|error| AuthError::network(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_codes_drive_ui_branches() {
        assert!(AuthError::new("xsts_2148916233", "no xbox").is_xsts());
        assert!(!AuthError::new("invalid_grant", "expired").is_xsts());
        assert!(AuthError::new("invalid_grant", "expired").is_invalid_grant());
        assert_eq!(AuthError::configuration("x").code, "configuration_error");
        assert_eq!(
            AuthError::rejected(None, None).code,
            "invalid_response".to_string()
        );
    }

    #[test]
    fn result_keeps_existing_fields_the_server_omits() {
        let mut profile = McProfile::authlib(
            "https://example.org/api/yggdrasil",
            "Example",
            "https://example.org",
            "meta",
            "token",
            "uuid",
            "Player",
        );
        profile.password = Some("secret".into());
        let result = AuthenticationResult {
            profile_type: ProfileType::Authlib,
            user_name: "Player2".into(),
            uuid: "uuid".into(),
            access_token: "token2".into(),
            refresh_token: String::new(),
            client_token: "client".into(),
            token_type: "Bearer".into(),
            expires_at: None,
            raw_json: String::new(),
            server: None,
            server_name: None,
            login_name: None,
            password: None,
            provider: Some("authlib".into()),
            discovery_address: None,
            id_token: None,
            xuid: None,
        };
        result.apply_to(&mut profile);
        assert_eq!(profile.access_token, "token2");
        assert_eq!(profile.user_name, "Player2");
        assert_eq!(profile.server_name.as_deref(), Some("Example"));
        assert_eq!(profile.password.as_deref(), Some("secret"));
        assert_eq!(profile.provider.as_deref(), Some("authlib"));
    }
}
