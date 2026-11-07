//! Profile storage and selection, mirroring `PCL.Core.Minecraft.Profile.ProfileService`
//! and `ProfileManagement`.
//!
//! CE 把档案列表与“上次使用”的下标存在配置里（`ProfileJson`）；Proto 用
//! [`crate::config::states::ProfileConfig`] 存同一结构，身份仍然是 `profile_id`。
//! 所有写入都走 `ConfigManager::update`，保存失败时内存状态保持不变。
//!
//! 这些函数会阻塞式地锁住配置，必须放在 `spawn_blocking` 里调用。

use crate::config::manager::ConfigManager;
use pcl_core::minecraft::profile::auth::{AuthError, AuthResult, AuthenticationResult};
use pcl_core::minecraft::profile::{McProfile, ProfileType};

/// 全部档案，按用户添加顺序。
pub fn all() -> Vec<McProfile> {
    ConfigManager::instance()
        .app_state
        .blocking_lock()
        .profile
        .profiles
        .clone()
}

/// 一次加锁读出档案列表与当前选择的 ID，避免两次读取之间被别的线程改动。
pub fn snapshot() -> (Vec<McProfile>, Option<String>) {
    let state = ConfigManager::instance().app_state.blocking_lock();
    let current = state
        .profile
        .current()
        .map(|profile| profile.profile_id.clone());
    (state.profile.profiles.clone(), current)
}

/// 当前选中的档案。
pub fn current() -> Option<McProfile> {
    let state = ConfigManager::instance().app_state.blocking_lock();
    state.profile.current().cloned()
}

pub fn find(profile_id: &str) -> Option<McProfile> {
    all()
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
}

/// 新增档案；`profile_id` 为空时生成一个新 ID，重复 ID 直接报错（对照 CE 的 `Add`）。
pub fn add(mut profile: McProfile, select: bool) -> AuthResult<McProfile> {
    if profile.profile_id.trim().is_empty() {
        profile.profile_id = new_profile_id();
    }
    let id = profile.profile_id.clone();
    ConfigManager::instance()
        .update(|state| {
            if state
                .profile
                .profiles
                .iter()
                .any(|existing| existing.profile_id == id)
            {
                return Err(format!("A profile with id '{id}' already exists."));
            }
            state.profile.profiles.push(profile.clone());
            if select {
                state.profile.select(Some(state.profile.profiles.len() - 1));
            }
            Ok(())
        })
        .map_err(|error| AuthError::new("profile_store", error))?;
    Ok(profile)
}

/// 用新内容替换同 ID 的档案（对照 CE 的 `Update`）。
pub fn replace(profile_id: &str, mut updated: McProfile) -> AuthResult<McProfile> {
    if updated.profile_id.trim().is_empty() {
        updated.profile_id = profile_id.to_owned();
    }
    let target = updated.profile_id.clone();
    let replacement = updated.clone();
    ConfigManager::instance()
        .update(|state| {
            let index = state
                .profile
                .profiles
                .iter()
                .position(|profile| profile.profile_id == profile_id)
                .ok_or("The source profile does not exist.")?;
            if state
                .profile
                .profiles
                .iter()
                .enumerate()
                .any(|(other, profile)| other != index && profile.profile_id == target)
            {
                return Err(format!("A profile with id '{target}' already exists."));
            }
            // CE 的 `Update` 保留 `lastUsed`，替换内容不影响当前选择。
            state.profile.profiles[index] = replacement.clone();
            Ok(())
        })
        .map_err(|error| AuthError::new("profile_store", error))?;
    Ok(updated)
}

/// 删除档案；删除的是当前档案时清空选择（对照 CE 的 `Delete`）。
pub fn remove(profile_id: &str) -> AuthResult<McProfile> {
    let id = profile_id.to_owned();
    ConfigManager::instance()
        .update(|state| {
            let index = state
                .profile
                .profiles
                .iter()
                .position(|profile| profile.profile_id == id)
                .ok_or("The source profile does not exist.")?;
            let removed = state.profile.profiles.remove(index);
            // CE 的 `Delete`：删掉当前档案时清空选择，删掉前面的档案时下标前移。
            if index as i32 == state.profile.last_used {
                state.profile.select(None);
            } else if (index as i32) < state.profile.last_used {
                state.profile.last_used -= 1;
            }
            Ok(removed)
        })
        .map_err(|error| AuthError::new("profile_store", error))
}

/// 选择档案；`None` 表示取消选择（对照 CE 的 `Select`）。
pub fn select(profile_id: Option<&str>) -> AuthResult<Option<McProfile>> {
    let id = profile_id.map(str::to_owned);
    ConfigManager::instance()
        .update(|state| {
            let index = match &id {
                None => None,
                Some(id) => Some(
                    state
                        .profile
                        .profiles
                        .iter()
                        .position(|profile| &profile.profile_id == id)
                        .ok_or("The source profile does not exist.")?,
                ),
            };
            state.profile.select(index);
            Ok(index.map(|index| state.profile.profiles[index].clone()))
        })
        .map_err(|error| AuthError::new("profile_store", error))
}

/// 新建档案用的随机 ID（对照 CE 的 `Guid.NewGuid().ToString("N")`）。
pub fn new_profile_id() -> String {
    uuid::Uuid::new_v4().simple().to_string()
}

/// 认证结果写回档案：已有档案就地更新，否则新增（对照 CE 的 `ApplyAuthenticationResult`）。
pub fn apply(
    result: &AuthenticationResult,
    existing: Option<&McProfile>,
    select: bool,
) -> AuthResult<McProfile> {
    match existing {
        Some(existing) => {
            let mut profile = existing.clone();
            result.apply_to(&mut profile);
            let updated = replace(&existing.profile_id, profile)?;
            if select {
                self::select(Some(&updated.profile_id))?;
            }
            Ok(updated)
        }
        None => {
            let mut profile = McProfile {
                profile_id: new_profile_id(),
                ..Default::default()
            };
            result.apply_to(&mut profile);
            add(profile, select)
        }
    }
}

/// 按类型 + UUID（第三方还要看服务器）找到已存在的同一账号，对照 CE 的 `_FindMatchingProfile`。
pub fn find_matching(result: &AuthenticationResult) -> Option<McProfile> {
    all().into_iter().find(|profile| {
        profile.profile_type == result.profile_type
            && profile.uuid == result.uuid
            && (result.profile_type == ProfileType::Microsoft
                || profile
                    .server
                    .as_deref()
                    .zip(result.server.as_deref())
                    .is_some_and(|(left, right)| left.eq_ignore_ascii_case(right)))
    })
}

/// 档案读写会阻塞式加锁，必须离开异步上下文。
pub async fn blocking<T: Send + 'static>(
    run: impl FnOnce() -> AuthResult<T> + Send + 'static,
) -> AuthResult<T> {
    tokio::task::spawn_blocking(run)
        .await
        .map_err(|error| AuthError::new("profile_store", error.to_string()))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_profiles_require_the_same_server_for_third_party() {
        let result = AuthenticationResult {
            profile_type: ProfileType::Authlib,
            user_name: "Player".into(),
            uuid: "uuid".into(),
            access_token: "token".into(),
            refresh_token: String::new(),
            client_token: String::new(),
            token_type: "Bearer".into(),
            expires_at: None,
            server: Some("https://a.example.org/api/yggdrasil/authserver".into()),
            server_name: None,
            login_name: None,
            password: None,
            discovery_address: None,
            xuid: None,
        };
        let mut same_server = McProfile::authlib(
            "https://a.example.org/api/yggdrasil/authserver",
            "A",
            "token",
            "uuid",
            "Player",
        );
        assert!(matches_profile(&result, &same_server));
        same_server.server = Some("https://b.example.org/api/yggdrasil/authserver".into());
        assert!(!matches_profile(&result, &same_server));
        // 微软账号不看服务器地址。
        let microsoft = AuthenticationResult {
            profile_type: ProfileType::Microsoft,
            server: None,
            ..result
        };
        let mut profile = McProfile::microsoft("Player", "uuid", "token", "refresh", "xuid");
        assert!(matches_profile(&microsoft, &profile));
        profile.uuid = "other".into();
        assert!(!matches_profile(&microsoft, &profile));
    }

    /// `find_matching` 的判定逻辑，脱离全局配置以便单测。
    fn matches_profile(result: &AuthenticationResult, profile: &McProfile) -> bool {
        profile.profile_type == result.profile_type
            && profile.uuid == result.uuid
            && (result.profile_type == ProfileType::Microsoft
                || profile
                    .server
                    .as_deref()
                    .zip(result.server.as_deref())
                    .is_some_and(|(left, right)| left.eq_ignore_ascii_case(right)))
    }

    #[test]
    fn results_only_overwrite_fields_the_server_provided() {
        let mut profile = McProfile::authlib(
            "https://a.example.org/api/yggdrasil/authserver",
            "A",
            "old",
            "uuid",
            "OldName",
        );
        let result = AuthenticationResult {
            profile_type: ProfileType::Authlib,
            user_name: "NewName".into(),
            uuid: "uuid".into(),
            access_token: "new".into(),
            refresh_token: String::new(),
            client_token: "client".into(),
            token_type: "Bearer".into(),
            expires_at: None,
            server: None,
            server_name: Some("Renamed".into()),
            login_name: None,
            password: None,
            discovery_address: None,
            xuid: None,
        };
        result.apply_to(&mut profile);
        assert_eq!(profile.user_name, "NewName");
        assert_eq!(profile.access_token, "new");
        assert_eq!(
            profile.server.as_deref(),
            Some("https://a.example.org/api/yggdrasil/authserver")
        );
    }
}
