//! 登录流程编排：复用、刷新与重新认证，对照 CE 的 `ProfileService.AuthenticateAsync`。

use super::{LoginRequest, Ui, cancelled};
use crate::minecraft::profile::auth::{
    AuthError, AuthResult, AuthenticationResult, authlib, connect, microsoft,
};
use crate::minecraft::profile::service::blocking;
use crate::minecraft::profile::{McProfile, ProfileType, service};

/// 完整的登录流程，对照 CE 的 `ProfileService.AuthenticateAsync`。
pub(super) async fn login(request: LoginRequest, ui: Ui) -> AuthResult<McProfile> {
    let profile_type = request.profile_type;
    if profile_type == ProfileType::Offline {
        return Err(AuthError::configuration("离线档案不需要远程认证"));
    }
    let profile_id = request.profile_id.clone();
    let existing = match profile_id {
        Some(id) => blocking(move || Ok(service::find(&id)))
            .await?
            .filter(|profile| profile.profile_type == profile_type),
        None => None,
    };
    let interactive = ui.session().is_some();

    // 1. 令牌还没过期：直接复用（CE 的短路分支）。
    if let Some(profile) = &existing {
        if !request.force_refresh
            && matches!(
                profile_type,
                ProfileType::Microsoft | ProfileType::YggdrasilConnect
            )
            && !profile.is_expired()
            && !profile.access_token.is_empty()
        {
            return select_and_return(profile).await;
        }
        // 2. 第三方令牌先做一次校验，有效就不用重新登录；校验失败只记日志，接着尝试刷新。
        if profile_type == ProfileType::Authlib
            && !request.force_refresh
            && !request.force_reselect_profile
        {
            let valid = match authlib_provider(profile) {
                Ok(provider) => {
                    provider
                        .validate(&profile.access_token, &profile.client_token)
                        .await
                }
                Err(error) => {
                    log::warn!("验证第三方档案失败，将尝试刷新：{error}");
                    false
                }
            };
            if valid {
                return select_and_return(profile).await;
            }
        }
    }

    // 3. 尝试刷新已有档案。
    let mut refreshed = None;
    if let Some(profile) = &existing
        && !request.force_refresh
        && (!request.force_reselect_profile || profile_type != ProfileType::Authlib)
    {
        match refresh(profile, &request, &ui).await {
            Ok(result) => refreshed = Some(result),
            Err(error) => {
                if error.is_xsts() {
                    return Err(error);
                }
                log::warn!("刷新档案失败，将尝试重新认证：{error}");
                // CE：只有刷新令牌失效（invalid_grant）才继续走完整登录，
                // 其它错误在没有界面时直接失败，避免启动流程静默降级。
                let has_credentials = profile_type == ProfileType::Authlib
                    && profile
                        .login_name
                        .as_deref()
                        .is_some_and(|name| !name.is_empty())
                    && profile
                        .password
                        .as_deref()
                        .is_some_and(|password| !password.is_empty());
                if !interactive && !error.is_invalid_grant() && !has_credentials {
                    return Err(error);
                }
            }
        }
    }

    // 4. 重新认证。
    let result = match refreshed {
        Some(result) => result,
        None => match profile_type {
            ProfileType::Microsoft => microsoft_login(&ui).await?,
            ProfileType::Authlib => authlib_login(&request, &ui).await?,
            ProfileType::YggdrasilConnect => {
                connect_login(&request, existing.as_ref(), &ui).await?
            }
            ProfileType::Offline => return Err(AuthError::configuration("离线档案不需要远程认证")),
        },
    };

    if ui.cancelled() {
        return Err(cancelled());
    }
    blocking(move || {
        let matched = existing.or_else(|| service::find_matching(&result));
        service::apply(&result, matched.as_ref(), true)
    })
    .await
}

async fn select_and_return(profile: &McProfile) -> AuthResult<McProfile> {
    let id = profile.profile_id.clone();
    let profile = profile.clone();
    blocking(move || {
        service::select(Some(&id))?;
        Ok(profile)
    })
    .await
}

/// 启动前的档案认证：不做任何交互，需要用户介入时返回错误。
pub async fn login_for_launch(profile: &McProfile, force_refresh: bool) -> AuthResult<McProfile> {
    let request = LoginRequest {
        profile_type: profile.profile_type,
        server: profile.server.clone(),
        discovery_address: profile.discovery_address.clone(),
        client_id: None,
        username: profile.login_name.clone(),
        password: profile.password.clone(),
        profile_id: Some(profile.profile_id.clone()),
        force_refresh,
        force_reselect_profile: false,
    };
    login(request, Ui::Headless).await
}

/// 强制换一份新的微软令牌并写回档案。
///
/// 皮肤上传这类接口可能在令牌“未过期但已被服务器拒绝”时返回 401，
/// CE 的做法是重跑登录加载器；这里直接刷新，避免在没有界面时要求重新登录。
pub async fn refresh_microsoft(profile: &McProfile) -> AuthResult<McProfile> {
    let provider = microsoft::MicrosoftProvider::new()?;
    let oauth = provider.refresh_oauth(&profile.refresh_token).await?;
    let result = provider.complete(&oauth).await?;
    let existing = profile.clone();
    blocking(move || service::apply(&result, Some(&existing), true)).await
}

async fn microsoft_login(ui: &Ui) -> AuthResult<AuthenticationResult> {
    let provider = microsoft::MicrosoftProvider::new()?;
    let data = provider.device_code().await?;
    let oauth = ui
        .device_code(&data, || provider.poll_device_code(&data))
        .await?;
    provider.complete(&oauth).await
}

async fn authlib_login(request: &LoginRequest, ui: &Ui) -> AuthResult<AuthenticationResult> {
    let server = request
        .server
        .as_deref()
        .filter(|server| !server.trim().is_empty())
        .ok_or_else(|| AuthError::configuration("第三方登录需要填写认证服务器地址"))?;
    let username = request.username.clone().unwrap_or_default();
    let password = request.password.clone().unwrap_or_default();
    let provider = authlib_provider_for_server(server).await?;
    match provider
        .login(&username, &password, request.force_reselect_profile)
        .await?
    {
        authlib::Login::Finished(result) => Ok(*result),
        authlib::Login::SelectProfile {
            candidates,
            access_token,
            client_token,
        } => {
            let Some(candidate) = ui.select_profile(candidates).await? else {
                return Err(cancelled());
            };
            provider
                .select(
                    &access_token,
                    &client_token,
                    &candidate,
                    &username,
                    &password,
                )
                .await
        }
    }
}

async fn connect_login(
    request: &LoginRequest,
    existing: Option<&McProfile>,
    ui: &Ui,
) -> AuthResult<AuthenticationResult> {
    let discovery = request
        .discovery_address
        .as_deref()
        .filter(|address| !address.trim().is_empty())
        .ok_or_else(|| AuthError::configuration("Yggdrasil Connect 缺少发现地址"))?
        .to_owned();
    let client = connect::Connect::new(
        &discovery,
        request.client_id.as_deref(),
        request.server.as_deref(),
    )
    .await?;
    let data = client.device_code().await?;
    let oauth = ui
        .device_code(&data, || client.poll_device_code(&data))
        .await?;
    match client
        .complete(&oauth, existing, false, request.force_reselect_profile)
        .await?
    {
        connect::Login::Finished(result) => Ok(*result),
        connect::Login::SelectProfile { candidates, oauth } => {
            let Some(candidate) = ui.select_profile(candidates).await? else {
                return Err(cancelled());
            };
            client.select(&oauth, &candidate).await
        }
    }
}

/// 刷新已有档案，对照 CE 各 Provider 的 `RefreshAsync`。
async fn refresh(
    profile: &McProfile,
    request: &LoginRequest,
    ui: &Ui,
) -> AuthResult<AuthenticationResult> {
    match profile.profile_type {
        ProfileType::Microsoft => {
            let provider = microsoft::MicrosoftProvider::new()?;
            let oauth = provider.refresh_oauth(&profile.refresh_token).await?;
            provider.complete(&oauth).await
        }
        ProfileType::Authlib => {
            let provider = authlib_provider(profile)?;
            provider.refresh(profile).await
        }
        ProfileType::YggdrasilConnect => {
            let discovery = profile.discovery_address.clone().ok_or_else(|| {
                AuthError::configuration("Yggdrasil Connect profile has no discovery address.")
            })?;
            let client = connect::Connect::new(
                &discovery,
                request.client_id.as_deref(),
                profile.server.as_deref(),
            )
            .await?;
            let oauth = client.refresh_oauth(&profile.refresh_token).await?;
            match client.complete(&oauth, Some(profile), true, false).await? {
                connect::Login::Finished(result) => Ok(*result),
                connect::Login::SelectProfile { candidates, oauth } => {
                    // 刷新时优先沿用档案里的角色；对不上才要求用户重新选择。
                    match candidates
                        .iter()
                        .find(|candidate| candidate.id == profile.uuid)
                    {
                        Some(candidate) => client.select(&oauth, candidate).await,
                        None => {
                            let Some(candidate) = ui.select_profile(candidates).await? else {
                                return Err(cancelled());
                            };
                            client.select(&oauth, &candidate).await
                        }
                    }
                }
            }
        }
        ProfileType::Offline => Err(AuthError::configuration(
            "Profile type 'offline' does not require remote authentication.",
        )),
    }
}

fn authlib_provider(profile: &McProfile) -> AuthResult<authlib::AuthlibProvider> {
    authlib::AuthlibProvider::new(
        profile
            .server
            .as_deref()
            .filter(|server| !server.trim().is_empty())
            .ok_or_else(|| AuthError::configuration("Authlib profile has no server."))?,
    )
}

/// 登录前跟随 `X-Authlib-Injector-Api-Location`，对照 CE 的 `ApiLocation.TryRequestAsync`。
async fn authlib_provider_for_server(server: &str) -> AuthResult<authlib::AuthlibProvider> {
    let client = crate::minecraft::profile::auth::http_client()?;
    let api_root = authlib::resolve_api_location(&client, server).await;
    authlib::AuthlibProvider::new(&api_root)
}
