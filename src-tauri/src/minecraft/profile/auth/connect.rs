//! Yggdrasil Connect provider, mirroring `YggdrasilConnectProvider`.
//!
//! 协议本质是 OpenID Connect 的设备码流程：登录结果放在 ID 令牌的
//! `selectedProfile` / `availableProfiles` 声明里，令牌签名用服务器的 JWKS 验证。

use chrono::Utc;
use jsonwebtoken::{Algorithm, DecodingKey, Validation, decode, decode_header, jwk::JwkSet};
use serde::Deserialize;
use serde_json::Value;

use super::oauth::{AuthorizeResult, DeviceCodeData, post_form};
use super::yggdrasil::YggdrasilProfile;
use super::{AuthError, AuthResult, AuthenticationResult, Candidate};
use crate::minecraft::profile::{McProfile, ProfileType};

/// CE 的 `_BuiltInClientIds`。
const BUILT_IN_CLIENT_IDS: &[(&str, &str)] = &[("littleskin.cn", "1514")];
/// 设备码流程申请的 scope，对照 CE 的 `_Scopes`。
const SCOPES: &str = "openid offline_access Yggdrasil.PlayerProfiles.Select Yggdrasil.Server.Join";
/// 元数据必须支持的 scope，对照 CE 的 `_scopesRequired`。
const REQUIRED_SCOPES: [&str; 3] = [
    "openid",
    "Yggdrasil.PlayerProfiles.Select",
    "Yggdrasil.Server.Join",
];

/// OpenID 发现文档，对照 CE 的 `YggdrasilConnectMetaData`。
#[derive(Debug, Deserialize)]
struct Metadata {
    issuer: String,
    token_endpoint: String,
    #[serde(default)]
    device_authorization_endpoint: Option<String>,
    #[serde(default)]
    user_info_endpoint: Option<String>,
    jwks_uri: String,
    #[serde(default)]
    scopes_supported: Vec<String>,
    #[serde(default)]
    id_token_signing_alg_values_supported: Vec<String>,
    #[serde(default)]
    shared_client_id: Option<String>,
}

/// 已校验的发现结果与解析出的客户端 ID。
#[derive(Clone, Debug)]
pub struct Discovery {
    pub address: String,
    pub issuer: String,
    pub token_endpoint: String,
    pub device_endpoint: String,
    pub user_info_endpoint: String,
    pub jwks_uri: String,
    pub client_id: String,
    pub signing_algorithms: Vec<String>,
}

/// ID 令牌与用户信息端点共用的声明，对照 CE 的 `YggdrasilConnectClaims`。
#[derive(Debug, Deserialize)]
struct Claims {
    sub: String,
    aud: Value,
    #[serde(default)]
    iat: Option<i64>,
    #[serde(rename = "selectedProfile", default)]
    selected_profile: Option<YggdrasilProfile>,
    #[serde(rename = "availableProfiles", default)]
    available_profiles: Vec<YggdrasilProfile>,
}

/// 登录中间态：多档案时先让用户选角色。
#[derive(Clone, Debug)]
pub enum Login {
    Finished(Box<AuthenticationResult>),
    SelectProfile {
        candidates: Vec<Candidate>,
        oauth: AuthorizeResult,
    },
}

pub fn built_in_client_id(host: &str) -> Option<&'static str> {
    let normalized = host.trim().trim_end_matches('.');
    BUILT_IN_CLIENT_IDS
        .iter()
        .find(|(candidate, _)| candidate.eq_ignore_ascii_case(normalized))
        .map(|(_, client_id)| *client_id)
}

/// authlib-injector 元数据里的 OpenID 发现地址。
pub fn discovery_address_of(metadata: &Value) -> Option<String> {
    metadata
        .pointer("/meta/feature.openid_configuration_url")
        .and_then(Value::as_str)
        .or_else(|| {
            metadata
                .get("feature.openid_configuration_url")
                .and_then(Value::as_str)
        })
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
}

/// 拉取并校验发现文档，对照 CE 的 `YggdrasilOptions.InitializeAsync`。
pub async fn initialize(
    client: &reqwest::Client,
    address: &str,
    client_id: Option<&str>,
) -> AuthResult<Discovery> {
    let url = reqwest::Url::parse(address).map_err(|_| {
        AuthError::configuration("Yggdrasil Connect discovery address must use HTTPS.")
    })?;
    if url.scheme() != "https" {
        return Err(AuthError::configuration(
            "Yggdrasil Connect discovery address must use HTTPS.",
        ));
    }
    let response = client.get(address).send().await?;
    let status = response.status();
    let text = response.text().await?;
    if !status.is_success() {
        return Err(AuthError::network(format!("{address} 返回 HTTP {status}")));
    }
    let metadata: Metadata = serde_json::from_str(&text)
        .map_err(|_| AuthError::configuration("无法加载 Yggdrasil Connect 元数据"))?;

    let issuer = reqwest::Url::parse(&metadata.issuer).ok();
    if issuer.as_ref().is_none_or(|url| url.scheme() != "https") {
        return Err(AuthError::configuration(
            "Yggdrasil Connect issuer must use HTTPS.",
        ));
    }
    let authority = |url: &reqwest::Url| {
        format!(
            "{}://{}{}",
            url.scheme(),
            url.host_str().unwrap_or_default(),
            url.port()
                .map(|port| format!(":{port}"))
                .unwrap_or_default()
        )
    };
    if issuer
        .as_ref()
        .is_none_or(|issuer| !authority(issuer).eq_ignore_ascii_case(&authority(&url)))
    {
        return Err(AuthError::configuration(
            "Yggdrasil Connect discovery address must belong to the issuer.",
        ));
    }
    for endpoint in [
        &metadata.token_endpoint,
        &metadata.jwks_uri,
        metadata
            .device_authorization_endpoint
            .as_deref()
            .unwrap_or(""),
        metadata.user_info_endpoint.as_deref().unwrap_or(""),
    ] {
        if endpoint.is_empty() {
            continue;
        }
        let parsed = reqwest::Url::parse(endpoint).ok();
        if parsed.as_ref().is_none_or(|url| url.scheme() != "https") {
            return Err(AuthError::configuration(format!(
                "Yggdrasil Connect endpoint must use HTTPS: {endpoint}"
            )));
        }
    }
    let Some(device_endpoint) = metadata.device_authorization_endpoint else {
        return Err(AuthError::configuration(
            "Yggdrasil Connect 元数据缺少设备授权端点",
        ));
    };
    let missing: Vec<&str> = REQUIRED_SCOPES
        .iter()
        .filter(|scope| !metadata.scopes_supported.iter().any(|it| it == **scope))
        .copied()
        .collect();
    if !missing.is_empty() {
        return Err(AuthError::configuration(format!(
            "Yggdrasil Connect 元数据缺少必要 scope：{}",
            missing.join(", ")
        )));
    }
    let resolved_client_id = client_id
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_owned)
        .or_else(|| metadata.shared_client_id.clone())
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| {
            AuthError::configuration(
                "Yggdrasil Connect 需要设置 ClientId，或由元数据提供 shared_client_id",
            )
        })?;
    Ok(Discovery {
        address: address.to_owned(),
        issuer: metadata.issuer,
        token_endpoint: metadata.token_endpoint,
        device_endpoint,
        user_info_endpoint: metadata.user_info_endpoint.unwrap_or_default(),
        jwks_uri: metadata.jwks_uri,
        client_id: resolved_client_id,
        signing_algorithms: metadata.id_token_signing_alg_values_supported,
    })
}

/// 已初始化的 Yggdrasil Connect 客户端。
pub struct Connect {
    client: reqwest::Client,
    discovery: Discovery,
    /// 用户填写的认证服务器地址；缺省时从发现地址推导。
    server: Option<String>,
}

impl Connect {
    pub async fn new(
        discovery_address: &str,
        client_id: Option<&str>,
        server: Option<&str>,
    ) -> AuthResult<Self> {
        let client = super::http_client()?;
        // CE 的优先级：调用方指定的 ClientId → 按服务器主机名匹配的内置 ID → 发现文档的 shared_client_id。
        let client_id = client_id
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .or_else(|| {
                reqwest::Url::parse(server.unwrap_or(discovery_address))
                    .ok()
                    .and_then(|url| {
                        url.host_str()
                            .and_then(built_in_client_id)
                            .map(str::to_owned)
                    })
            });
        let discovery = initialize(&client, discovery_address, client_id.as_deref()).await?;
        Ok(Self {
            client,
            discovery,
            server: server.map(str::to_owned),
        })
    }

    /// 存进档案 `server` 字段的地址，对照 CE 的 `_GetServerAddress`。
    pub fn server_address(&self) -> String {
        if let Some(server) = self
            .server
            .as_deref()
            .map(str::trim)
            .filter(|it| !it.is_empty())
        {
            let server = server.trim_end_matches('/');
            return if server.ends_with("/authserver") {
                server.to_owned()
            } else {
                format!("{server}/authserver")
            };
        }
        let value = &self.discovery.address;
        let marker = value.to_ascii_lowercase().find("/.well-known/");
        match marker {
            Some(index) if index > 0 => value[..index].trim_end_matches('/').to_owned(),
            _ => value.trim_end_matches('/').to_owned(),
        }
    }

    pub async fn device_code(&self) -> AuthResult<DeviceCodeData> {
        let data: DeviceCodeData = post_form(
            &self.client,
            &self.discovery.device_endpoint,
            &[
                ("scope", SCOPES),
                ("client_id", self.discovery.client_id.as_str()),
            ],
        )
        .await?;
        if let Some(error) = data.error() {
            return Err(error);
        }
        data.validate()?;
        Ok(data)
    }

    pub async fn poll_device_code(&self, data: &DeviceCodeData) -> AuthResult<AuthorizeResult> {
        let result: AuthorizeResult = post_form(
            &self.client,
            &self.discovery.token_endpoint,
            &[
                ("client_id", self.discovery.client_id.as_str()),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
                (
                    "device_code",
                    data.device_code.as_deref().unwrap_or_default(),
                ),
            ],
        )
        .await?;
        result.validate(true, false)?;
        Ok(result)
    }

    /// 静默刷新（`grant_type=refresh_token`），对照 CE 的 `AuthorizeWithSilentAsync`。
    pub async fn refresh_oauth(&self, refresh_token: &str) -> AuthResult<AuthorizeResult> {
        if refresh_token.trim().is_empty() {
            return Err(AuthError::rejected(
                Some("invalid_grant".into()),
                Some("刷新令牌为空，需要重新登录。".into()),
            ));
        }
        let result: AuthorizeResult = post_form(
            &self.client,
            &self.discovery.token_endpoint,
            &[
                ("refresh_token", refresh_token),
                ("grant_type", "refresh_token"),
                ("client_id", self.discovery.client_id.as_str()),
            ],
        )
        .await?;
        if let Some(error) = result.error() {
            return Err(error);
        }
        result.validate(true, false)?;
        Ok(result)
    }

    /// 令牌换玩家档案，对照 CE 的 `_CompleteAsync`。
    pub async fn complete(
        &self,
        oauth: &AuthorizeResult,
        existing: Option<&McProfile>,
        silent_refresh: bool,
        force_reselect: bool,
    ) -> AuthResult<Login> {
        if let Some(error) = oauth.error() {
            return Err(error);
        }
        oauth.validate(true, !silent_refresh)?;
        let claims = self
            .read_id_token(oauth.id_token.as_deref().unwrap_or_default())
            .await?;
        let mut profile = claims.selected_profile.clone();
        let mut available = claims.available_profiles.clone();
        if profile.is_none() || available.is_empty() {
            let user_info = self
                .user_info(oauth.access_token.as_deref().unwrap_or_default())
                .await;
            if let Some(info) = user_info {
                if profile.is_none() {
                    profile = info.selected_profile;
                }
                if available.is_empty() {
                    available = info.available_profiles;
                }
            }
        }
        if profile.is_none()
            && !force_reselect
            && existing.is_some_and(|existing| !existing.uuid.is_empty())
        {
            let uuid = existing
                .map(|existing| existing.uuid.as_str())
                .unwrap_or_default();
            profile = available
                .iter()
                .find(|candidate| candidate.id == uuid)
                .cloned();
        }
        if profile.is_none()
            && available.is_empty()
            && let Some(existing) = existing
            && !existing.uuid.is_empty()
            && !existing.user_name.is_empty()
        {
            profile = Some(YggdrasilProfile {
                id: existing.uuid.clone(),
                name: existing.user_name.clone(),
            });
        }
        // CE：强制重选，或服务器没有指定当前角色且可选角色多于一个时，交给用户选择。
        if (force_reselect || profile.is_none()) && available.len() > 1 {
            return Ok(Login::SelectProfile {
                candidates: super::authlib::candidates(&available),
                oauth: oauth.clone(),
            });
        }
        let Some(profile) = profile.or_else(|| available.first().cloned()) else {
            return Err(AuthError::rejected(
                Some("invalid_profile".into()),
                Some("Yggdrasil Connect returned no player profile.".into()),
            ));
        };
        Ok(Login::Finished(Box::new(
            self.finish(oauth, &profile).await?,
        )))
    }

    /// 用户选定角色后完成登录。
    pub async fn select(
        &self,
        oauth: &AuthorizeResult,
        candidate: &Candidate,
    ) -> AuthResult<AuthenticationResult> {
        let profile = YggdrasilProfile {
            id: candidate.id.clone(),
            name: candidate.name.clone(),
        };
        self.finish(oauth, &profile).await
    }

    async fn finish(
        &self,
        oauth: &AuthorizeResult,
        profile: &YggdrasilProfile,
    ) -> AuthResult<AuthenticationResult> {
        let expires_in = oauth.expires_in.unwrap_or_default();
        let server_name = self.server_name().await;
        Ok(AuthenticationResult {
            profile_type: ProfileType::YggdrasilConnect,
            user_name: profile.name.clone(),
            uuid: profile.id.clone(),
            access_token: oauth.access_token.clone().unwrap_or_default(),
            refresh_token: oauth.refresh_token.clone().unwrap_or_default(),
            client_token: profile.id.clone(),
            token_type: oauth.token_type.clone().unwrap_or_else(|| "Bearer".into()),
            expires_at: (expires_in > 0)
                .then(|| Utc::now() + chrono::Duration::seconds(expires_in)),
            server: Some(self.server_address()),
            server_name,
            login_name: None,
            password: None,
            discovery_address: Some(self.discovery.address.clone()),
            xuid: None,
        })
    }

    /// 读取认证服务器根地址的 `meta.serverName`。
    async fn server_name(&self) -> Option<String> {
        let root = self.server_address();
        let root = root
            .strip_suffix("/authserver")
            .unwrap_or(&root)
            .trim_end_matches('/');
        super::authlib::server_name(&self.client, root).await
    }

    /// 校验并解析 ID 令牌，对照 CE 的 `_ReadIdTokenAsync`。
    async fn read_id_token(&self, id_token: &str) -> AuthResult<Claims> {
        if id_token.trim().is_empty() {
            return Err(AuthError::new(
                "invalid_token",
                "Yggdrasil Connect did not return an ID token.",
            ));
        }
        let header = decode_header(id_token)
            .map_err(|error| AuthError::new("invalid_token", format!("令牌解析失败：{error}")))?;
        let kid = header.kid.ok_or_else(|| {
            AuthError::new("invalid_token", "Yggdrasil Connect ID token has no key id.")
        })?;
        let algorithm = self.signing_algorithm(header.alg)?;
        let jwks: JwkSet = self
            .client
            .get(&self.discovery.jwks_uri)
            .send()
            .await?
            .json()
            .await
            .map_err(|error| AuthError::new("invalid_token", format!("JWKS 读取失败：{error}")))?;
        let key = jwks
            .keys
            .iter()
            .find(|key| key.common.key_id.as_deref() == Some(kid.as_str()))
            .ok_or_else(|| AuthError::configuration(format!("找不到匹配的 Jwk：{kid}")))?;
        let decoding = DecodingKey::from_jwk(key)
            .map_err(|error| AuthError::new("invalid_token", format!("Jwk 无效：{error}")))?;
        let mut validation = Validation::new(algorithm);
        validation.set_issuer(&[&self.discovery.issuer]);
        validation.set_audience(&[&self.discovery.client_id]);
        validation.leeway = 60;
        let token = decode::<Claims>(id_token, &decoding, &validation).map_err(|error| {
            AuthError::new(
                "invalid_token",
                format!("Yggdrasil Connect ID token validation failed: {error}"),
            )
        })?;
        // CE 额外拒绝签发时间在未来的令牌。
        if let Some(iat) = token.claims.iat
            && iat > (Utc::now() + chrono::Duration::seconds(60)).timestamp()
        {
            return Err(AuthError::new(
                "invalid_token",
                "Token issued-at time is in the future.",
            ));
        }
        if token.claims.sub.is_empty() || audience_is_empty(&token.claims.aud) {
            return Err(AuthError::new(
                "invalid_token",
                "Yggdrasil Connect ID token payload was empty.",
            ));
        }
        Ok(token.claims)
    }

    /// 只接受发现文档声明的非对称签名算法。
    ///
    /// 对称算法（HS*）一律拒绝：JWKS 里只有公钥，接受 HMAC 会让任何人都能伪造令牌。
    fn signing_algorithm(&self, algorithm: Algorithm) -> AuthResult<Algorithm> {
        let name = algorithm_name(algorithm);
        if !self.discovery.signing_algorithms.is_empty()
            && !self
                .discovery
                .signing_algorithms
                .iter()
                .any(|supported| supported == name)
        {
            return Err(AuthError::new(
                "invalid_token",
                format!("服务器不支持的 ID 令牌签名算法：{name}"),
            ));
        }
        match algorithm {
            Algorithm::RS256
            | Algorithm::RS384
            | Algorithm::RS512
            | Algorithm::ES256
            | Algorithm::ES384
            | Algorithm::EdDSA => Ok(algorithm),
            other => Err(AuthError::new(
                "invalid_token",
                format!("不支持的 ID 令牌签名算法：{other:?}"),
            )),
        }
    }

    /// 用户信息端点，作为 ID 令牌声明不全时的回退，对照 CE 的 `_GetUserInfoAsync`。
    async fn user_info(&self, access_token: &str) -> Option<UserInfo> {
        if self.discovery.user_info_endpoint.is_empty() {
            return None;
        }
        let response = self
            .client
            .get(&self.discovery.user_info_endpoint)
            .bearer_auth(access_token)
            .send()
            .await
            .ok()?;
        if !response.status().is_success() {
            return None;
        }
        // 声明不完整时按“未提供用户信息”处理，交由后续回退逻辑。
        response.json().await.ok()
    }
}

#[derive(Debug, Deserialize)]
struct UserInfo {
    #[serde(rename = "selectedProfile", default)]
    selected_profile: Option<YggdrasilProfile>,
    #[serde(rename = "availableProfiles", default)]
    available_profiles: Vec<YggdrasilProfile>,
}

/// `jsonwebtoken` 的 `Algorithm` 没有 `Display`，这里给出发现文档里使用的名字。
fn algorithm_name(algorithm: Algorithm) -> &'static str {
    match algorithm {
        Algorithm::RS256 => "RS256",
        Algorithm::RS384 => "RS384",
        Algorithm::RS512 => "RS512",
        Algorithm::ES256 => "ES256",
        Algorithm::ES384 => "ES384",
        Algorithm::EdDSA => "EdDSA",
        _ => "unknown",
    }
}

fn audience_is_empty(audience: &Value) -> bool {
    match audience {
        Value::String(value) => value.is_empty(),
        Value::Array(values) => values.is_empty(),
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn built_in_client_ids_match_ce() {
        assert_eq!(built_in_client_id("littleskin.cn"), Some("1514"));
        assert_eq!(built_in_client_id("LittleSkin.cn."), Some("1514"));
        assert_eq!(built_in_client_id("example.org"), None);
    }

    #[test]
    fn discovery_address_comes_from_authlib_metadata() {
        let metadata = serde_json::json!({
            "meta": { "serverName": "LittleSkin", "feature.openid_configuration_url": "https://auth.littleskin.cn/.well-known/openid-configuration" }
        });
        assert_eq!(
            discovery_address_of(&metadata).as_deref(),
            Some("https://auth.littleskin.cn/.well-known/openid-configuration")
        );
        assert_eq!(discovery_address_of(&serde_json::json!({})), None);
    }

    #[test]
    fn server_address_falls_back_to_discovery_authority() {
        let discovery = Discovery {
            address: "https://auth.littleskin.cn/.well-known/openid-configuration".into(),
            issuer: "https://auth.littleskin.cn".into(),
            token_endpoint: "https://auth.littleskin.cn/oauth/token".into(),
            device_endpoint: "https://auth.littleskin.cn/oauth/device".into(),
            user_info_endpoint: String::new(),
            jwks_uri: "https://auth.littleskin.cn/oauth/jwks".into(),
            client_id: "1514".into(),
            signing_algorithms: vec![],
        };
        let with_server = Connect {
            client: reqwest::Client::new(),
            discovery: discovery.clone(),
            server: Some("https://littleskin.cn/api/yggdrasil".into()),
        };
        assert_eq!(
            with_server.server_address(),
            "https://littleskin.cn/api/yggdrasil/authserver"
        );
        let without_server = Connect {
            client: reqwest::Client::new(),
            discovery,
            server: None,
        };
        assert_eq!(
            without_server.server_address(),
            "https://auth.littleskin.cn"
        );
    }

    /// 真实皮肤站冒烟测试：LittleSkin 的 authlib 元数据 + Yggdrasil Connect 发现文档。
    #[tokio::test]
    #[ignore = "需要网络与真实认证服务器"]
    async fn littleskin_supports_device_flow() {
        let client = super::super::http_client().unwrap();
        let api_root = super::super::authlib::resolve_api_location(
            &client,
            "https://littleskin.cn/api/yggdrasil",
        )
        .await;
        let api_root = super::super::authlib::normalize_server(&api_root).unwrap();
        assert_eq!(api_root, "https://littleskin.cn/api/yggdrasil");
        assert_eq!(
            super::super::authlib::server_name(&client, &api_root)
                .await
                .as_deref(),
            Some("LittleSkin")
        );
        let response = client.get(&api_root).send().await.unwrap();
        let metadata: serde_json::Value = response.json().await.unwrap();
        let discovery = super::discovery_address_of(&metadata).expect("缺少 OpenID 发现地址");
        // 真实登录路径由服务器主机名解析内置 ClientId（元数据没有 shared_client_id）。
        let host = reqwest::Url::parse(&api_root)
            .unwrap()
            .host_str()
            .unwrap()
            .to_owned();
        let built_in = super::built_in_client_id(&host);
        assert_eq!(built_in, Some("1514"));
        let resolved = super::initialize(&client, &discovery, built_in)
            .await
            .unwrap();
        assert_eq!(resolved.client_id, "1514", "内置 ClientId 应生效");
        assert!(!resolved.device_endpoint.is_empty());
        assert!(!resolved.token_endpoint.is_empty());
        assert!(!resolved.jwks_uri.is_empty());

        // 设备码请求本身也要能成功（不完成登录，用户码随后过期）。
        let connect = super::Connect::new(&discovery, built_in, Some(&api_root))
            .await
            .unwrap();
        assert_eq!(
            connect.server_address(),
            "https://littleskin.cn/api/yggdrasil/authserver"
        );
        let code = connect.device_code().await.unwrap();
        assert!(!code.user_code.unwrap_or_default().is_empty());
        assert!(!code.verification_uri.unwrap_or_default().is_empty());
    }

    #[test]
    fn claims_parse_selected_and_available_profiles() {
        let claims: Claims = serde_json::from_value(serde_json::json!({
            "sub": "1",
            "aud": "1514",
            "selectedProfile": { "id": "uuid", "name": "Player" },
            "availableProfiles": [{ "id": "uuid", "name": "Player" }]
        }))
        .unwrap();
        assert_eq!(claims.selected_profile.unwrap().name, "Player");
        assert_eq!(claims.available_profiles.len(), 1);
        assert!(!audience_is_empty(&serde_json::json!("1514")));
        assert!(audience_is_empty(&serde_json::json!([])));
    }
}
