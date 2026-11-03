//! Legacy Yggdrasil (authlib-injector) HTTP client, mirroring
//! `PCL.Core.Minecraft.IdentityModel.Yggdrasil.YggdrasilLegacyClient`.

use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use super::{AuthError, AuthResult};

/// 认证服务器返回的玩家档案。
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct YggdrasilProfile {
    pub id: String,
    #[serde(default)]
    pub name: String,
}

/// `authenticate` / `refresh` 的响应，对照 CE 的 `YggdrasilAuthenticateResult`。
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticateResponse {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_message: Option<String>,
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub client_token: Option<String>,
    #[serde(default)]
    pub selected_profile: Option<YggdrasilProfile>,
    #[serde(default)]
    pub available_profiles: Option<Vec<YggdrasilProfile>>,
}

impl AuthenticateResponse {
    pub fn is_error(&self) -> bool {
        self.error.as_deref().is_some_and(|error| !error.is_empty())
    }

    /// 服务器把失败原因放在 `error` / `errorMessage` 里。
    pub fn error(&self) -> Option<AuthError> {
        self.is_error()
            .then(|| AuthError::rejected(self.error.clone(), self.error_message.clone()))
    }
}

/// 用户名密码登录（`/authserver/authenticate`）。
pub async fn authenticate(
    client: &reqwest::Client,
    api_root: &str,
    username: &str,
    password: &str,
    client_token: Option<&str>,
) -> AuthResult<AuthenticateResponse> {
    post(
        client,
        &format!("{api_root}/authserver/authenticate"),
        &json!({
            "username": username,
            "password": password,
            "agent": { "name": "Minecraft", "version": 1 },
            "requestUser": false,
            "clientToken": client_token.unwrap_or_default(),
        }),
    )
    .await
}

/// 刷新令牌（`/authserver/refresh`），`selected` 用于在多档案时切换角色。
pub async fn refresh(
    client: &reqwest::Client,
    api_root: &str,
    access_token: &str,
    client_token: Option<&str>,
    selected: Option<&YggdrasilProfile>,
) -> AuthResult<AuthenticateResponse> {
    let mut body = json!({
        "accessToken": access_token,
        "requestUser": false,
    });
    if let Some(client_token) = client_token
        && !client_token.is_empty()
    {
        body["clientToken"] = Value::String(client_token.to_owned());
    }
    if let Some(selected) = selected {
        body["selectedProfile"] = json!({ "id": selected.id, "name": selected.name });
    }
    post(client, &format!("{api_root}/authserver/refresh"), &body).await
}

/// 校验访问令牌是否仍然有效（`/authserver/validate`，成功返回 204）。
pub async fn validate(
    client: &reqwest::Client,
    api_root: &str,
    access_token: &str,
    client_token: Option<&str>,
) -> bool {
    let body = json!({
        "accessToken": access_token,
        "clientToken": client_token.unwrap_or_default(),
    });
    match client
        .post(format!("{api_root}/authserver/validate"))
        .json(&body)
        .send()
        .await
    {
        Ok(response) => response.status() == reqwest::StatusCode::NO_CONTENT,
        Err(_) => false,
    }
}

async fn post(
    client: &reqwest::Client,
    url: &str,
    body: &Value,
) -> AuthResult<AuthenticateResponse> {
    let response = client.post(url).json(body).send().await?;
    let status = response.status();
    let text = response.text().await?;
    serde_json::from_str(&text).map_err(|_| {
        AuthError::network(format!(
            "{url} 返回 HTTP {status}，且响应不是 Yggdrasil JSON：{}",
            text.chars().take(200).collect::<String>()
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_errors_keep_their_code_and_message() {
        let response: AuthenticateResponse = serde_json::from_value(json!({
            "error": "ForbiddenOperationException",
            "errorMessage": "密码不正确"
        }))
        .unwrap();
        let error = response.error().unwrap();
        assert_eq!(error.code, "ForbiddenOperationException");
        assert_eq!(error.message, "密码不正确");
    }
}
