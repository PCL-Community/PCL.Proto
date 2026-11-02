//! OAuth 2.0 token and device-authorization primitives, mirroring
//! `PCL.Core.Minecraft.IdentityModel.OAuth`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{AuthError, AuthResult};

/// Token endpoint response, mirroring `AuthorizeResult`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AuthorizeResult {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
    #[serde(default)]
    pub access_token: Option<String>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub id_token: Option<String>,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub expires_in: Option<i64>,
    #[serde(default)]
    pub scope: Option<String>,
}

impl AuthorizeResult {
    pub fn is_error(&self) -> bool {
        self.error.as_deref().is_some_and(|error| !error.is_empty())
    }

    /// 按 OAuth 2.0 / Yggdrasil Connect 规范校验令牌响应的必须字段。
    ///
    /// 对照 CE 的 `AuthorizeResult.Validate`：错误响应交给调用方处理，不在此校验。
    pub fn validate(&self, require_id_token: bool, require_refresh_token: bool) -> AuthResult<()> {
        if self.is_error() {
            return Ok(());
        }
        let required = |value: Option<&str>, field: &str| {
            if value.is_none_or(str::is_empty) {
                return Err(AuthError::rejected(
                    Some("invalid_request".into()),
                    Some(format!("令牌响应缺少必须字段 {field}")),
                ));
            }
            Ok(())
        };
        required(self.access_token.as_deref(), "access_token")?;
        required(self.token_type.as_deref(), "token_type")?;
        if !self
            .token_type
            .as_deref()
            .is_some_and(|kind| kind.eq_ignore_ascii_case("Bearer"))
        {
            return Err(AuthError::rejected(
                Some("invalid_request".into()),
                Some(format!(
                    "不支持的 token_type：{}",
                    self.token_type.clone().unwrap_or_default()
                )),
            ));
        }
        if self.expires_in.is_none_or(|seconds| seconds < 0) {
            return Err(AuthError::rejected(
                Some("invalid_request".into()),
                Some("令牌响应缺少必须字段 expires_in".into()),
            ));
        }
        if require_refresh_token && self.refresh_token.as_deref().is_none_or(str::is_empty) {
            return Err(AuthError::rejected(
                Some("invalid_request".into()),
                Some("已申请 offline_access 但令牌响应缺少 refresh_token".into()),
            ));
        }
        if require_id_token && self.id_token.as_deref().is_none_or(str::is_empty) {
            return Err(AuthError::rejected(
                Some("invalid_request".into()),
                Some("已申请 openid 但令牌响应缺少 id_token".into()),
            ));
        }
        Ok(())
    }

    /// 把错误响应转成 [`AuthError`]，非错误响应返回 `None`。
    pub fn error(&self) -> Option<AuthError> {
        self.is_error()
            .then(|| AuthError::rejected(self.error.clone(), self.error_description.clone()))
    }
}

/// Device authorization response, mirroring `DeviceCodeData` (RFC 8628).
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct DeviceCodeData {
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub error_description: Option<String>,
    #[serde(default)]
    pub user_code: Option<String>,
    #[serde(default)]
    pub device_code: Option<String>,
    #[serde(default)]
    pub verification_uri: Option<String>,
    #[serde(default)]
    pub verification_uri_complete: Option<String>,
    #[serde(default)]
    pub interval: Option<u64>,
    #[serde(default)]
    pub expires_in: Option<i64>,
}

impl DeviceCodeData {
    pub fn is_error(&self) -> bool {
        self.error.as_deref().is_some_and(|error| !error.is_empty())
    }

    /// 对照 CE 的 `DeviceCodeData.Validate`。
    pub fn validate(&self) -> AuthResult<()> {
        if self.is_error() {
            return Ok(());
        }
        for (value, field) in [
            (self.device_code.as_deref(), "device_code"),
            (self.user_code.as_deref(), "user_code"),
            (self.verification_uri.as_deref(), "verification_uri"),
        ] {
            if value.is_none_or(str::is_empty) {
                return Err(AuthError::rejected(
                    Some("invalid_request".into()),
                    Some(format!("设备授权响应缺少必须字段 {field}")),
                ));
            }
        }
        if self.expires_in.is_none_or(|seconds| seconds < 0) {
            return Err(AuthError::rejected(
                Some("invalid_request".into()),
                Some("设备授权响应缺少必须字段 expires_in".into()),
            ));
        }
        Ok(())
    }

    pub fn error(&self) -> Option<AuthError> {
        self.is_error()
            .then(|| AuthError::rejected(self.error.clone(), self.error_description.clone()))
    }

    /// 轮询间隔，CE 默认 5 秒。
    pub fn poll_interval(&self) -> u64 {
        self.interval.unwrap_or(5).max(1)
    }
}

/// 一次设备码轮询的结果，对照 CE `MyMsgLogin.WorkThreadAsync` 的分支。
#[derive(Clone, Debug)]
pub enum PollOutcome {
    /// `authorization_pending`：按当前间隔继续等待。
    Pending,
    /// `slow_down`：间隔增加 5 秒后继续等待。
    SlowDown,
    /// 用户已授权，拿到令牌。
    Authorized(AuthorizeResult),
    /// 设备码过期、用户拒绝或服务器返回其它错误，登录结束。
    Rejected(AuthError),
}

/// 把令牌端点的错误码归类为轮询结果。
pub fn classify(result: AuthorizeResult) -> PollOutcome {
    if let Some(error) = result.error() {
        return match error.code.as_str() {
            "authorization_pending" => PollOutcome::Pending,
            "slow_down" => PollOutcome::SlowDown,
            "expired_token" => PollOutcome::Rejected(AuthError::new(
                error.code,
                fallback(error.message, "设备授权已过期。"),
            )),
            "access_denied" | "authorization_declined" => PollOutcome::Rejected(AuthError::new(
                error.code,
                fallback(error.message, "用户拒绝了设备授权。"),
            )),
            _ => PollOutcome::Rejected(error),
        };
    }
    PollOutcome::Authorized(result)
}

/// 服务器没有给出描述时使用 CE 的默认文案。
fn fallback(message: String, default: &str) -> String {
    if message.trim().is_empty() {
        default.into()
    } else {
        message
    }
}

/// 解析 JWT 的 payload 段，不做签名校验。
///
/// 只用于读取微软令牌里的 `xuid` 这类展示性声明；需要防伪造的 Yggdrasil Connect
/// ID 令牌走 [`super::connect`] 的签名验证。
pub fn jwt_payload(token: &str) -> AuthResult<Value> {
    use base64::Engine;
    let payload = token
        .split('.')
        .nth(1)
        .ok_or_else(|| AuthError::new("invalid_token", "令牌不是有效的 JWT"))?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|error| AuthError::new("invalid_token", error.to_string()))?;
    serde_json::from_slice(&bytes)
        .map_err(|error| AuthError::new("invalid_token", error.to_string()))
}

/// POST 一个表单并把响应体解析为 JSON。
///
/// OAuth 的错误同样以 JSON 响应体返回（通常带 400），所以这里不按状态码抛错，
/// 交给调用方读取 `error` 字段。
pub async fn post_form(
    client: &reqwest::Client,
    url: &str,
    form: &[(&str, &str)],
) -> AuthResult<Value> {
    let response = client
        .post(url)
        .form(
            &form
                .iter()
                .map(|(key, value)| (*key, *value))
                .collect::<Vec<_>>(),
        )
        .send()
        .await?;
    let status = response.status();
    let text = response.text().await?;
    serde_json::from_str(&text).map_err(|_| {
        AuthError::network(format!(
            "{url} 返回 HTTP {status}，且响应不是 JSON：{}",
            text.chars().take(200).collect::<String>()
        ))
    })
}

/// POST 一个表单并解析为令牌响应。
pub async fn post_form_tokens(
    client: &reqwest::Client,
    url: &str,
    form: &[(&str, &str)],
) -> AuthResult<AuthorizeResult> {
    let value = post_form(client, url, form).await?;
    serde_json::from_value(value).map_err(|error| AuthError::network(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tokens(error: &str) -> AuthorizeResult {
        AuthorizeResult {
            error: Some(error.into()),
            ..Default::default()
        }
    }

    #[test]
    fn poll_errors_map_onto_ce_branches() {
        assert!(matches!(
            classify(tokens("authorization_pending")),
            PollOutcome::Pending
        ));
        assert!(matches!(
            classify(tokens("slow_down")),
            PollOutcome::SlowDown
        ));
        match classify(tokens("expired_token")) {
            PollOutcome::Rejected(error) => assert_eq!(error.code, "expired_token"),
            other => panic!("unexpected {other:?}"),
        }
        match classify(tokens("access_denied")) {
            PollOutcome::Rejected(error) => assert_eq!(error.code, "access_denied"),
            other => panic!("unexpected {other:?}"),
        }
        match classify(tokens("server_error")) {
            PollOutcome::Rejected(error) => assert_eq!(error.code, "server_error"),
            other => panic!("unexpected {other:?}"),
        }
    }

    #[test]
    fn token_response_requires_protocol_fields() {
        let incomplete = AuthorizeResult {
            access_token: Some("token".into()),
            ..Default::default()
        };
        assert_eq!(
            incomplete.validate(false, false).unwrap_err().code,
            "invalid_request"
        );

        let unsupported = AuthorizeResult {
            access_token: Some("token".into()),
            token_type: Some("mac".into()),
            expires_in: Some(60),
            ..Default::default()
        };
        assert!(unsupported.validate(false, false).is_err());

        let openid = AuthorizeResult {
            access_token: Some("token".into()),
            token_type: Some("Bearer".into()),
            expires_in: Some(60),
            ..Default::default()
        };
        assert!(openid.validate(false, false).is_ok());
        assert_eq!(
            openid.validate(true, true).unwrap_err().message,
            "已申请 offline_access 但令牌响应缺少 refresh_token"
        );
        assert!(
            openid
                .validate(true, false)
                .unwrap_err()
                .message
                .contains("id_token")
        );
    }

    #[test]
    fn device_code_response_requires_protocol_fields() {
        let incomplete = DeviceCodeData {
            user_code: Some("CODE".into()),
            ..Default::default()
        };
        assert!(incomplete.validate().is_err());
        let complete = DeviceCodeData {
            device_code: Some("device".into()),
            user_code: Some("CODE".into()),
            verification_uri: Some("https://example.org/device".into()),
            expires_in: Some(900),
            interval: None,
            ..Default::default()
        };
        assert!(complete.validate().is_ok());
        assert_eq!(complete.poll_interval(), 5);
        // 错误响应不做字段校验，交给调用方处理。
        assert!(
            DeviceCodeData {
                error: Some("access_denied".into()),
                ..Default::default()
            }
            .validate()
            .is_ok()
        );
    }
}
