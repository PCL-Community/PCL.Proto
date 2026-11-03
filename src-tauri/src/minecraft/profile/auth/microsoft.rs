//! Microsoft consumer account provider, mirroring `MicrosoftProvider`.
//!
//! OAuth → Xbox Live → XSTS → Minecraft 的六步协议全部留在这里，启动流程只需要
//! 最终的 [`AuthenticationResult`]。

use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};

use super::oauth::{AuthorizeResult, DeviceCodeData, jwt_payload, post_form_tokens};
use super::{AuthError, AuthResult, AuthenticationResult};
use crate::minecraft::profile::ProfileType;

const DEVICE_ENDPOINT: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/devicecode";
const TOKEN_ENDPOINT: &str = "https://login.microsoftonline.com/consumers/oauth2/v2.0/token";
const XBOX_ENDPOINT: &str = "https://user.auth.xboxlive.com/user/authenticate";
const XSTS_ENDPOINT: &str = "https://xsts.auth.xboxlive.com/xsts/authorize";
const MINECRAFT_LOGIN_ENDPOINT: &str =
    "https://api.minecraftservices.com/authentication/login_with_xbox";
const MINECRAFT_LICENSE_ENDPOINT: &str = "https://api.minecraftservices.com/entitlements/mcstore";
const MINECRAFT_PROFILE_ENDPOINT: &str = "https://api.minecraftservices.com/minecraft/profile";
const SCOPE: &str = "XboxLive.signin offline_access";

/// `login_with_xbox` 的响应，对照 CE 的 `MojangResponse`。
#[derive(Debug, Deserialize)]
struct MinecraftToken {
    access_token: String,
    #[serde(default)]
    expires_in: i64,
    #[serde(default = "default_token_type")]
    token_type: String,
}

fn default_token_type() -> String {
    "Bearer".into()
}

/// `minecraft/profile` 的响应，对照 CE 的 `MojangProfile`。
#[derive(Debug, Deserialize)]
struct MinecraftProfile {
    id: String,
    name: String,
    /// CE 把整个档案响应存进 `RawJson`。
    raw_json: String,
}

/// Xbox Live / XSTS 的响应，对照 CE 的 `XboxLiveResponse`。
#[derive(Debug, Deserialize)]
struct XboxResponse {
    #[serde(rename = "Token")]
    token: String,
    #[serde(rename = "DisplayClaims")]
    display_claims: Option<XboxDisplayClaims>,
}

#[derive(Debug, Deserialize)]
struct XboxDisplayClaims {
    #[serde(default)]
    xui: Vec<XboxUserHash>,
}

#[derive(Debug, Deserialize)]
struct XboxUserHash {
    #[serde(default)]
    uhs: Option<String>,
}

pub struct MicrosoftProvider {
    client: reqwest::Client,
    client_id: String,
}

impl MicrosoftProvider {
    /// 使用配置的 Client ID，未配置时直接报配置错误（CE 同样不接受空配置）。
    pub fn new() -> AuthResult<Self> {
        let client_id = crate::app::secrets::ms_oauth_client_id().ok_or_else(|| {
            AuthError::configuration(
                "未配置微软 OAuth 的 Client ID：请在构建时设置 MS_CLIENT_ID，或通过 .env / 运行环境变量提供",
            )
        })?;
        Ok(Self {
            client: super::http_client()?,
            client_id,
        })
    }

    /// 申请设备码，对照 CE 的 `GetDeviceCodeAsync`。
    pub async fn device_code(&self) -> AuthResult<DeviceCodeData> {
        let value = super::oauth::post_form(
            &self.client,
            DEVICE_ENDPOINT,
            &[("client_id", self.client_id.as_str()), ("scope", SCOPE)],
        )
        .await?;
        let data: DeviceCodeData =
            serde_json::from_value(value).map_err(|error| AuthError::network(error.to_string()))?;
        if let Some(error) = data.error() {
            return Err(error);
        }
        data.validate()?;
        Ok(data)
    }

    /// 轮询一次设备码，对照 CE 的 `PollDeviceCodeAsync`。
    pub async fn poll_device_code(&self, data: &DeviceCodeData) -> AuthResult<AuthorizeResult> {
        let result = post_form_tokens(
            &self.client,
            TOKEN_ENDPOINT,
            &[
                ("client_id", self.client_id.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                (
                    "device_code",
                    data.device_code.as_deref().unwrap_or_default(),
                ),
            ],
        )
        .await?;
        result.validate(false, false)?;
        Ok(result)
    }

    /// 用刷新令牌换取新的 OAuth 令牌，对照 CE 的 `RefreshOAuthAsync`。
    pub async fn refresh_oauth(&self, refresh_token: &str) -> AuthResult<AuthorizeResult> {
        if refresh_token.trim().is_empty() {
            return Err(AuthError::rejected(
                Some("invalid_grant".into()),
                Some("Microsoft refresh token is empty.".into()),
            ));
        }
        let result = post_form_tokens(
            &self.client,
            TOKEN_ENDPOINT,
            &[
                ("client_id", self.client_id.as_str()),
                ("grant_type", "refresh_token"),
                ("refresh_token", refresh_token),
                ("scope", SCOPE),
            ],
        )
        .await?;
        if let Some(error) = result.error() {
            return Err(error);
        }
        result.validate(false, false)?;
        Ok(AuthorizeResult {
            refresh_token: result.refresh_token.or_else(|| Some(refresh_token.into())),
            ..result
        })
    }

    /// OAuth 令牌之后的 Xbox / Minecraft 步骤，对照 CE 的 `CompleteAsync`。
    pub async fn complete(&self, oauth: &AuthorizeResult) -> AuthResult<AuthenticationResult> {
        if let Some(error) = oauth.error() {
            return Err(error);
        }
        let access_token = oauth.access_token.as_deref().unwrap_or_default();
        if access_token.is_empty() {
            return Err(AuthError::rejected(
                Some("invalid_token".into()),
                Some("Microsoft OAuth did not return an access token.".into()),
            ));
        }

        let xbl = self
            .xbox_authenticate(
                json!({
                    "Properties": {
                        "AuthMethod": "RPS",
                        "SiteName": "user.auth.xboxlive.com",
                        "RpsTicket": format!("d={access_token}")
                    },
                    "RelyingParty": "http://auth.xboxlive.com",
                    "TokenType": "JWT"
                }),
                false,
            )
            .await?;
        let xsts = self
            .xbox_authenticate(
                json!({
                    "Properties": {
                        "SandboxId": "RETAIL",
                        "UserTokens": [xbl.token]
                    },
                    "RelyingParty": "rp://api.minecraftservices.com/",
                    "TokenType": "JWT"
                }),
                true,
            )
            .await?;
        let user_hash = xsts
            .display_claims
            .as_ref()
            .and_then(|claims| claims.xui.first())
            .and_then(|user| user.uhs.clone())
            .filter(|hash| !hash.is_empty())
            .ok_or_else(|| {
                AuthError::rejected(
                    Some("xsts_missing_user_hash".into()),
                    Some("XSTS response did not contain a user hash.".into()),
                )
            })?;

        let minecraft = self.minecraft_login(&xsts.token, &user_hash).await?;
        if minecraft.access_token.is_empty() {
            return Err(AuthError::rejected(
                Some("minecraft_authentication_failed".into()),
                Some("Minecraft access token is empty.".into()),
            ));
        }
        if !self.owns_minecraft(&minecraft).await? {
            return Err(AuthError::rejected(
                Some("minecraft_not_owned".into()),
                Some("The Microsoft account does not own Minecraft.".into()),
            ));
        }
        let profile = self.minecraft_profile(&minecraft).await?;
        let expires_in = minecraft.expires_in;
        Ok(AuthenticationResult {
            profile_type: ProfileType::Microsoft,
            user_name: profile.name,
            uuid: profile.id.clone(),
            access_token: minecraft.access_token.clone(),
            refresh_token: oauth.refresh_token.clone().unwrap_or_default(),
            client_token: profile.id,
            token_type: minecraft.token_type,
            expires_at: (expires_in > 0)
                .then(|| Utc::now() + chrono::Duration::seconds(expires_in)),
            raw_json: profile.raw_json,
            server: None,
            server_name: None,
            login_name: None,
            password: None,
            provider: Some("microsoft".into()),
            discovery_address: None,
            id_token: None,
            // CE 不保存 xuid；Proto 的 ${auth_xuid} 启动参数需要它。
            xuid: jwt_payload(&minecraft.access_token)
                .ok()
                .and_then(|payload| payload.get("xuid")?.as_str().map(str::to_owned)),
        })
    }

    /// 修改正版玩家 ID，对照 CE 的 `ProfileUi.EditProfileId`。
    ///
    /// 失败时的 `code` 供界面映射到本地化文案：`duplicate_name`、`not_allowed`、
    /// `name_change_cooldown`。
    pub async fn change_name(&self, access_token: &str, name: &str) -> AuthResult<String> {
        let check = self
            .client
            .get(format!(
                "{MINECRAFT_PROFILE_ENDPOINT}/name/{name}/available"
            ))
            .bearer_auth(access_token)
            .send()
            .await?;
        let status = check.status();
        let text = check.text().await?;
        if !status.is_success() {
            return Err(AuthError::network(format!(
                "名称可用性检查返回 HTTP {status}：{}",
                text.chars().take(200).collect::<String>()
            )));
        }
        let value: Value =
            serde_json::from_str(&text).map_err(|error| AuthError::network(error.to_string()))?;
        match value.get("status").and_then(Value::as_str) {
            Some("DUPLICATE") => return Err(AuthError::new("duplicate_name", "该名称已被占用。")),
            Some("NOT_ALLOWED") => return Err(AuthError::new("not_allowed", "该名称不被允许。")),
            _ => {}
        }
        let response = self
            .client
            .put(format!("{MINECRAFT_PROFILE_ENDPOINT}/name/{name}"))
            .bearer_auth(access_token)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if status == reqwest::StatusCode::FORBIDDEN {
            return Err(AuthError::new(
                "name_change_cooldown",
                "改名过于频繁，请稍后再试。",
            ));
        }
        if !status.is_success() {
            return Err(AuthError::network(format!(
                "改名请求返回 HTTP {status}：{}",
                text.chars().take(200).collect::<String>()
            )));
        }
        let profile: MinecraftProfile =
            serde_json::from_str(&text).map_err(|error| AuthError::network(error.to_string()))?;
        Ok(profile.name)
    }

    async fn xbox_authenticate(&self, body: Value, is_xsts: bool) -> AuthResult<XboxResponse> {
        let url = if is_xsts {
            XSTS_ENDPOINT
        } else {
            XBOX_ENDPOINT
        };
        let response = self.client.post(url).json(&body).send().await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            if is_xsts {
                // XSTS 的账号级错误（未注册 Xbox、地区限制、未成年等）需要专门指引。
                if let Ok(error) = serde_json::from_str::<Value>(&text)
                    && let Some(code) = error.get("XErr").and_then(Value::as_i64)
                {
                    let message = error
                        .get("Message")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                        .unwrap_or_else(|| format!("XSTS authorization failed with XErr {code}."));
                    return Err(AuthError::rejected(
                        Some(format!("xsts_{code}")),
                        Some(message),
                    ));
                }
            }
            return Err(AuthError::network(format!(
                "{url} 返回 HTTP {status}：{}",
                text.chars().take(200).collect::<String>()
            )));
        }
        serde_json::from_str(&text).map_err(|error| AuthError::network(error.to_string()))
    }

    async fn minecraft_login(
        &self,
        xsts_token: &str,
        user_hash: &str,
    ) -> AuthResult<MinecraftToken> {
        let response = self
            .client
            .post(MINECRAFT_LOGIN_ENDPOINT)
            .json(&json!({
                "identityToken": format!("XBL3.0 x={user_hash};{xsts_token}")
            }))
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(AuthError::network(format!(
                "Minecraft 登录返回 HTTP {status}：{}",
                text.chars().take(200).collect::<String>()
            )));
        }
        serde_json::from_str(&text).map_err(|error| AuthError::network(error.to_string()))
    }

    /// 检查账号是否持有 Minecraft 许可，对照 CE 的 `MojangUtils.CheckLicenseAsync`。
    pub async fn owns_minecraft_with(
        &self,
        token_type: &str,
        access_token: &str,
    ) -> AuthResult<bool> {
        let scheme = if token_type.trim().is_empty() {
            "Bearer"
        } else {
            token_type
        };
        let response = self
            .client
            .get(MINECRAFT_LICENSE_ENDPOINT)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("{scheme} {access_token}"),
            )
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if !status.is_success() {
            return Err(AuthError::network(format!(
                "许可检查返回 HTTP {status}：{}",
                text.chars().take(200).collect::<String>()
            )));
        }
        let value: Value =
            serde_json::from_str(&text).map_err(|error| AuthError::network(error.to_string()))?;
        Ok(value
            .get("items")
            .and_then(Value::as_array)
            .is_some_and(|items| {
                items.iter().any(|item| {
                    matches!(
                        item.get("name").and_then(Value::as_str),
                        Some("product_minecraft" | "game_minecraft")
                    )
                })
            }))
    }

    async fn owns_minecraft(&self, token: &MinecraftToken) -> AuthResult<bool> {
        self.owns_minecraft_with(&token.token_type, &token.access_token)
            .await
    }

    /// 读取玩家档案，对照 CE 的 `MojangUtils.GetProfileAsync`。
    ///
    /// 返回 `(uuid, 名称, 原始 JSON)`；原始 JSON 会存进档案，供启动参数与皮肤使用。
    pub async fn profile_with(&self, access_token: &str) -> AuthResult<(String, String, String)> {
        let response = self
            .client
            .get(MINECRAFT_PROFILE_ENDPOINT)
            .bearer_auth(access_token)
            .send()
            .await?;
        let status = response.status();
        let text = response.text().await?;
        if status == reqwest::StatusCode::NOT_FOUND {
            return Err(AuthError::rejected(
                Some("profile_not_created".into()),
                Some("The Microsoft account has not created a Minecraft Java profile yet.".into()),
            ));
        }
        if !status.is_success() {
            return Err(AuthError::network(format!(
                "玩家档案返回 HTTP {status}：{}",
                text.chars().take(200).collect::<String>()
            )));
        }
        let profile: MinecraftProfile =
            serde_json::from_str(&text).map_err(|error| AuthError::network(error.to_string()))?;
        Ok((profile.id, profile.name, text))
    }

    async fn minecraft_profile(&self, token: &MinecraftToken) -> AuthResult<MinecraftProfile> {
        let (id, name, raw_json) = self.profile_with(&token.access_token).await?;
        Ok(MinecraftProfile { id, name, raw_json })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provider_requires_a_configured_client_id() {
        if crate::app::secrets::ms_oauth_client_id().is_some() {
            return;
        }
        match MicrosoftProvider::new() {
            Ok(_) => panic!("未配置 Client ID 时不应构造出 Provider"),
            Err(error) => assert_eq!(error.code, "configuration_error"),
        }
    }

    /// 真实微软端点冒烟测试：只申请设备码，不完成登录。
    #[tokio::test]
    #[ignore = "需要网络、微软端点与已配置的 MS_CLIENT_ID"]
    async fn device_code_endpoint_returns_a_user_code() {
        let provider = match MicrosoftProvider::new() {
            Ok(provider) => provider,
            Err(error) => {
                eprintln!("跳过：未配置可用于冒烟测试的 Client ID（{error}）");
                return;
            }
        };
        let data = provider.device_code().await.unwrap();
        assert!(!data.user_code.unwrap_or_default().is_empty());
        assert!(!data.verification_uri.unwrap_or_default().is_empty());
        assert!(data.expires_in.unwrap_or_default() > 0);
    }
}
