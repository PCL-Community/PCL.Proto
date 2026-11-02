//! authlib-injector provider, mirroring `PCL.Core.Minecraft.Profile.Authentication.AuthlibProvider`.

use serde_json::Value;

use super::oauth::jwt_payload;
use super::yggdrasil::{self, YggdrasilProfile};
use super::{AuthError, AuthResult, AuthenticationResult, Candidate};
use crate::minecraft::profile::ProfileType;

/// 用户名密码登录的中间态：多档案时需要用户先选角色。
#[derive(Clone, Debug)]
pub enum Login {
    Finished(Box<AuthenticationResult>),
    SelectProfile {
        candidates: Vec<Candidate>,
        access_token: String,
        client_token: String,
    },
}

pub struct AuthlibProvider {
    client: reqwest::Client,
    api_root: String,
    auth_server: String,
}

impl AuthlibProvider {
    /// `server` 允许带或不带 `/authserver` 后缀，对照 CE 的 `_NormalizeServer`。
    pub fn new(server: &str) -> AuthResult<Self> {
        let api_root = normalize_server(server)?;
        Ok(Self {
            client: super::http_client()?,
            auth_server: format!("{api_root}/authserver"),
            api_root,
        })
    }

    /// 用户名密码登录，对照 CE 的 `AuthlibProvider.AuthenticateAsync`。
    pub async fn login(
        &self,
        username: &str,
        password: &str,
        force_reselect: bool,
    ) -> AuthResult<Login> {
        if username.trim().is_empty() || password.is_empty() {
            return Err(AuthError::rejected(
                Some("invalid_credentials".into()),
                Some("请填写邮箱与密码。".into()),
            ));
        }
        let response =
            yggdrasil::authenticate(&self.client, &self.api_root, username, password, None).await?;
        if let Some(error) = response.error() {
            return Err(error);
        }
        let available = response.available_profiles.clone().unwrap_or_default();
        if !force_reselect && let Some(selected) = response.selected_profile.clone() {
            return Ok(Login::Finished(Box::new(
                self.finish(&response, &selected, username, password).await,
            )));
        }
        let Some(selected) = pick_single(&available) else {
            return Ok(Login::SelectProfile {
                candidates: candidates(&available),
                access_token: response.access_token.clone().unwrap_or_default(),
                client_token: response.client_token.clone().unwrap_or_default(),
            });
        };
        self.switch_to(&response, &selected, username, password)
            .await
    }

    /// 用户在多档案里选定角色后继续登录。
    pub async fn select(
        &self,
        access_token: &str,
        client_token: &str,
        candidate: &Candidate,
        username: &str,
        password: &str,
    ) -> AuthResult<AuthenticationResult> {
        let selected = YggdrasilProfile {
            id: candidate.id.clone(),
            name: candidate.name.clone(),
        };
        let response = yggdrasil::refresh(
            &self.client,
            &self.api_root,
            access_token,
            (!client_token.is_empty()).then_some(client_token),
            Some(&selected),
        )
        .await?;
        if let Some(error) = response.error() {
            return Err(error);
        }
        let selected = response.selected_profile.clone().unwrap_or(selected);
        Ok(self.finish(&response, &selected, username, password).await)
    }

    /// 用已保存的令牌刷新，对照 CE 的 `AuthlibProvider.RefreshAsync`。
    pub async fn refresh(
        &self,
        access_token: &str,
        client_token: &str,
        uuid: &str,
        user_name: &str,
        login_name: Option<&str>,
        password: Option<&str>,
    ) -> AuthResult<AuthenticationResult> {
        let selected = YggdrasilProfile {
            id: uuid.to_owned(),
            name: user_name.to_owned(),
        };
        let response = yggdrasil::refresh(
            &self.client,
            &self.api_root,
            access_token,
            (!client_token.is_empty()).then_some(client_token),
            Some(&selected),
        )
        .await?;
        if let Some(error) = response.error() {
            return Err(error);
        }
        let selected = response.selected_profile.clone().unwrap_or(selected);
        Ok(self
            .finish(
                &response,
                &selected,
                login_name.unwrap_or_default(),
                password.unwrap_or_default(),
            )
            .await)
    }

    /// 令牌是否仍然有效，对照 CE 的 `AuthlibProvider.ValidateAsync`。
    pub async fn validate(&self, access_token: &str, client_token: &str) -> bool {
        if access_token.trim().is_empty() {
            return false;
        }
        yggdrasil::validate(
            &self.client,
            &self.api_root,
            access_token,
            (!client_token.is_empty()).then_some(client_token),
        )
        .await
    }

    /// 读取服务器名称（`meta.serverName`），失败时返回 `None`，与 CE 一致。
    pub async fn server_name(&self) -> Option<String> {
        server_name(&self.client, &self.api_root).await
    }

    /// 首次登录后如果服务器默认角色与用户选择不一致，需要再刷新一次切换角色。
    async fn switch_to(
        &self,
        response: &yggdrasil::AuthenticateResponse,
        selected: &YggdrasilProfile,
        username: &str,
        password: &str,
    ) -> AuthResult<Login> {
        let access_token = response.access_token.clone().unwrap_or_default();
        let client_token = response.client_token.clone().unwrap_or_default();
        let switched = yggdrasil::refresh(
            &self.client,
            &self.api_root,
            &access_token,
            (!client_token.is_empty()).then_some(client_token.as_str()),
            Some(selected),
        )
        .await?;
        if let Some(error) = switched.error() {
            return Err(error);
        }
        let selected = switched
            .selected_profile
            .clone()
            .unwrap_or_else(|| selected.clone());
        Ok(Login::Finished(Box::new(
            self.finish(&switched, &selected, username, password).await,
        )))
    }

    async fn finish(
        &self,
        response: &yggdrasil::AuthenticateResponse,
        selected: &YggdrasilProfile,
        username: &str,
        password: &str,
    ) -> AuthenticationResult {
        let server_name = self.server_name().await;
        AuthenticationResult {
            profile_type: ProfileType::Authlib,
            user_name: selected.name.clone(),
            uuid: selected.id.clone(),
            access_token: response.access_token.clone().unwrap_or_default(),
            refresh_token: String::new(),
            client_token: response.client_token.clone().unwrap_or_default(),
            token_type: "Bearer".into(),
            expires_at: None,
            raw_json: response
                .selected_profile
                .as_ref()
                .and_then(|profile| serde_json::to_string(profile).ok())
                .unwrap_or_default(),
            server: Some(self.auth_server.clone()),
            server_name,
            login_name: (!username.is_empty()).then(|| username.to_owned()),
            password: (!password.is_empty()).then(|| password.to_owned()),
            provider: Some("authlib".into()),
            discovery_address: None,
            id_token: None,
            // 第三方服务器的 access token 同样是 JWT，xuid 声明可缺省。
            xuid: response
                .access_token
                .as_deref()
                .and_then(|token| jwt_payload(token).ok())
                .and_then(|payload| {
                    payload
                        .get("xuid")
                        .and_then(Value::as_str)
                        .map(str::to_owned)
                }),
        }
    }
}

/// 只有一个可选角色时直接选中，多个时交给界面询问，与 CE 的 `_SelectProfileAsync` 一致。
fn pick_single(available: &[YggdrasilProfile]) -> Option<YggdrasilProfile> {
    match available.len() {
        0 => None,
        1 => available.first().cloned(),
        _ => None,
    }
}

pub fn candidates(available: &[YggdrasilProfile]) -> Vec<Candidate> {
    available
        .iter()
        .map(|profile| Candidate {
            id: profile.id.clone(),
            name: if profile.name.is_empty() {
                profile.id.clone()
            } else {
                profile.name.clone()
            },
        })
        .collect()
}

/// 去掉用户输入里的 `/authserver` 后缀并要求是绝对地址。
pub fn normalize_server(server: &str) -> AuthResult<String> {
    let trimmed = server.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err(AuthError::configuration("第三方登录需要填写认证服务器地址"));
    }
    let value = trimmed.strip_suffix("/authserver").unwrap_or(trimmed);
    if reqwest::Url::parse(value).is_err() {
        return Err(AuthError::configuration(format!(
            "认证服务器地址无效：{value}"
        )));
    }
    Ok(value.to_owned())
}

/// 跟随 `X-Authlib-Injector-Api-Location` 头，对照 CE 的 `ApiLocation.TryRequestAsync`。
pub async fn resolve_api_location(client: &reqwest::Client, address: &str) -> String {
    let origin = if address.starts_with("http") {
        address.to_owned()
    } else {
        format!("https://{address}")
    };
    let Ok(origin_url) = reqwest::Url::parse(&origin) else {
        return origin;
    };
    let Ok(response) = client.head(&origin).send().await else {
        return origin;
    };
    let Some(location) = response
        .headers()
        .get("X-Authlib-Injector-Api-Location")
        .and_then(|value| value.to_str().ok())
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
    else {
        return origin;
    };
    if location.starts_with(origin_url.scheme()) {
        return location;
    }
    // 不允许 HTTPS 降级到 HTTP。
    if location.starts_with("http:") && origin_url.scheme() == "https" {
        return location.replacen("http", "https", 1);
    }
    origin_url
        .join(&location)
        .map(|url| url.to_string())
        .unwrap_or(origin)
}

/// 读取认证服务器根地址的 `meta.serverName`。
pub async fn server_name(client: &reqwest::Client, api_root: &str) -> Option<String> {
    let response = client.get(api_root).send().await.ok()?;
    if !response.status().is_success() {
        return None;
    }
    let metadata: Value = response.json().await.ok()?;
    metadata
        .pointer("/meta/serverName")
        .and_then(Value::as_str)
        .map(str::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn server_addresses_keep_ce_shape() {
        assert_eq!(
            normalize_server("https://littleskin.cn/api/yggdrasil/").unwrap(),
            "https://littleskin.cn/api/yggdrasil"
        );
        assert_eq!(
            normalize_server("https://littleskin.cn/api/yggdrasil/authserver").unwrap(),
            "https://littleskin.cn/api/yggdrasil"
        );
        assert!(normalize_server("littleskin.cn").is_err());
        let provider_root = normalize_server("https://example.org/api/yggdrasil").unwrap();
        assert_eq!(
            format!("{provider_root}/authserver"),
            "https://example.org/api/yggdrasil/authserver"
        );
    }

    /// 真实皮肤站冒烟测试：错误凭据要拿到服务器的错误码，而不是解析失败。
    #[tokio::test]
    #[ignore = "需要网络与真实认证服务器"]
    async fn wrong_credentials_return_the_server_error() {
        let provider = super::AuthlibProvider::new("https://littleskin.cn/api/yggdrasil").unwrap();
        let error = provider
            .login("pcl-proto-smoke@example.org", "not-the-password", false)
            .await
            .expect_err("错误凭据不应该登录成功");
        assert!(!error.code.is_empty());
        assert_ne!(
            error.code, "network_error",
            "应拿到认证服务器的错误：{error}"
        );
        // 服务器名称同样可读，档案列表会显示它。
        assert_eq!(provider.server_name().await.as_deref(), Some("LittleSkin"));
    }

    #[test]
    fn single_profile_is_selected_without_asking() {
        let one = vec![YggdrasilProfile {
            id: "uuid".into(),
            name: "Player".into(),
        }];
        assert_eq!(pick_single(&one).unwrap().id, "uuid");
        let many = vec![
            YggdrasilProfile {
                id: "a".into(),
                name: "A".into(),
            },
            YggdrasilProfile {
                id: "b".into(),
                name: "".into(),
            },
        ];
        assert!(pick_single(&many).is_none());
        assert!(pick_single(&[]).is_none());
        let names = candidates(&many);
        assert_eq!(names[1].name, "b", "空名字回退到 UUID");
    }
}
