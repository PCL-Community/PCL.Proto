//! Launcher profiles, mirroring `PCL.Core.Minecraft.Profile.Models`.
//!
//! The C# core keeps a mutable [`McProfile`] whose stable identity is the
//! profile id. Rust keeps the fields used by Proto and represents the profile kind with
//! an enum-derived `profile_type` and uses `Option` for values that are only
//! meaningful to one authentication provider.

use serde::{Deserialize, Serialize};

pub mod auth;
pub mod service;
pub mod uuid;

/// Authentication provider of a launcher profile.
///
/// Mirrors `PCL.Core.Minecraft.Profile.Models.ProfileType`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProfileType {
    #[serde(rename = "microsoft")]
    Microsoft,
    #[default]
    #[serde(rename = "offline")]
    Offline,
    #[serde(rename = "authlib")]
    Authlib,
    #[serde(rename = "yggdrasilConnect")]
    YggdrasilConnect,
}

/// Full persisted launcher profile.
///
/// Mirrors `PCL.Core.Minecraft.Profile.Models.McProfile` / `SafeProfile`.
/// Provider-only fields stay optional so offline profiles do not accumulate
/// meaningless credentials.
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct McProfile {
    pub profile_id: String,
    // CE 的 JSON 名：`username`（不是 `userName`）。
    #[serde(rename = "username")]
    pub user_name: String,
    pub access_token: String,
    pub uuid: String,
    pub token_type: String,
    pub profile_type: ProfileType,

    pub refresh_token: String,
    pub client_token: String,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub description: String,

    // Authlib / Yggdrasil
    pub server: Option<String>,
    pub server_name: Option<String>,
    // CE 的 JSON 名：`name`（属性名是 LoginName）。
    #[serde(rename = "name")]
    pub login_name: Option<String>,
    pub password: Option<String>,

    // Yggdrasil Connect discovery used when refreshing credentials.
    pub discovery_address: Option<String>,

    // Authentication details kept by the launcher for Xbox/Minecraft arguments.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub xuid: Option<String>,
}

impl Default for McProfile {
    fn default() -> Self {
        Self {
            profile_id: String::new(),
            user_name: String::new(),
            access_token: String::new(),
            uuid: String::new(),
            token_type: "Bearer".into(),
            profile_type: ProfileType::Offline,
            refresh_token: String::new(),
            client_token: String::new(),
            expires_at: None,
            description: String::new(),
            server: None,
            server_name: None,
            login_name: None,
            password: None,
            discovery_address: None,
            xuid: None,
        }
    }
}

impl McProfile {
    pub fn offline(user_name: impl Into<String>, uuid: impl Into<String>) -> Self {
        let user_name = user_name.into();
        let uuid = uuid.into();
        Self {
            profile_id: uuid.clone(),
            user_name,
            uuid,
            ..Self::default()
        }
    }

    pub fn yggdrasil(
        user_name: impl Into<String>,
        uuid: impl Into<String>,
        access_token: impl Into<String>,
    ) -> Self {
        let user_name = user_name.into();
        let uuid = uuid.into();
        Self {
            profile_id: uuid.clone(),
            user_name,
            uuid,
            access_token: access_token.into(),
            profile_type: ProfileType::YggdrasilConnect,
            ..Self::default()
        }
    }

    pub fn microsoft(
        user_name: impl Into<String>,
        uuid: impl Into<String>,
        access_token: impl Into<String>,
        refresh_token: impl Into<String>,
        xuid: impl Into<String>,
    ) -> Self {
        let user_name = user_name.into();
        let uuid = uuid.into();
        Self {
            profile_id: uuid.clone(),
            user_name,
            uuid,
            access_token: access_token.into(),
            refresh_token: refresh_token.into(),
            profile_type: ProfileType::Microsoft,
            xuid: Some(xuid.into()),
            ..Self::default()
        }
    }

    pub fn authlib(
        api_location: impl Into<String>,
        server_name: impl Into<String>,
        access_token: impl Into<String>,
        uuid: impl Into<String>,
        user_name: impl Into<String>,
    ) -> Self {
        let uuid = uuid.into();
        Self {
            profile_id: uuid.clone(),
            user_name: user_name.into(),
            uuid,
            access_token: access_token.into(),
            profile_type: ProfileType::Authlib,
            server: Some(api_location.into()),
            server_name: Some(server_name.into()),
            ..Self::default()
        }
    }

    pub fn access_token(&self) -> Option<&str> {
        (!self.access_token.is_empty()).then_some(self.access_token.as_str())
    }

    pub fn xuid(&self) -> Option<&str> {
        self.xuid.as_deref().filter(|value| !value.is_empty())
    }

    pub fn is_expired(&self) -> bool {
        self.expires_at
            .is_some_and(|expires| expires <= chrono::Utc::now() + chrono::Duration::minutes(1))
    }

    /// Validate the parts of a profile that are required to launch a game.
    pub fn validate_for_launch(&self) -> Result<(), String> {
        if self.user_name.trim().is_empty() {
            return Err("Profile has no player name".into());
        }
        if self.uuid.trim().is_empty() {
            return Err("Profile has no UUID".into());
        }
        // CE 的 IsProfileValid：名称里的引号会破坏启动参数。
        if self.user_name.contains('"') {
            return Err("Player name contains a quote".into());
        }
        if matches!(
            self.profile_type,
            ProfileType::Microsoft | ProfileType::YggdrasilConnect | ProfileType::Authlib
        ) && self.access_token.is_empty()
        {
            return Err("Profile has no access token".into());
        }
        Ok(())
    }
}

/// UI-safe projection of a profile, mirroring `PCL.Core.Minecraft.Profile.Models.SafeProfile`.
///
/// Tokens and provider secrets stay in the backend; the webview only receives
/// identity and display fields.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SafeProfile {
    pub profile_id: String,
    pub user_name: String,
    pub uuid: String,
    pub profile_type: ProfileType,
    pub description: String,
    pub server: Option<String>,
    pub server_name: Option<String>,
}

impl From<&McProfile> for SafeProfile {
    fn from(profile: &McProfile) -> Self {
        Self {
            profile_id: profile.profile_id.clone(),
            user_name: profile.user_name.clone(),
            uuid: profile.uuid.clone(),
            profile_type: profile.profile_type,
            description: profile.description.clone(),
            server: profile.server.clone(),
            server_name: profile.server_name.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_profile_never_exposes_tokens() {
        let profile = McProfile::microsoft("Player", "uuid", "access", "refresh", "xuid");
        let safe = SafeProfile::from(&profile);
        let value = serde_json::to_value(&safe).unwrap();
        assert_eq!(value["userName"], "Player");
        assert_eq!(value["profileType"], "microsoft");
        assert!(value.get("accessToken").is_none());
        assert!(value.get("refreshToken").is_none());
        assert!(value.get("xuid").is_none());
    }
}
