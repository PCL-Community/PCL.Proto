//! Build-time secrets, mirroring `PCL.Core.App.Secrets`.

/// 空白值按未配置处理。
fn usable_key(value: Option<String>) -> Option<String> {
    let value = value?.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

/// CurseForge API 的 Client ID。
///
/// 对照 CE：发布构建在编译期把 `CURSEFORGE_API_KEY` 烧进二进制（CE 用 Source Generator），
/// 只有开发构建才回退到运行环境变量；这里用 `option_env!` 复现同一优先级。
pub fn curseforge_api_key() -> Option<String> {
    usable_key(option_env!("CURSEFORGE_API_KEY").map(str::to_owned))
        .or_else(|| usable_key(std::env::var("CURSEFORGE_API_KEY").ok()))
        .or_else(|| usable_key(std::env::var("PCL_CURSEFORGE_API_KEY").ok()))
}

/// 微软 OAuth 的 Client ID，对照 CE 的 `Secrets.MSOAuthClientId`。
///
/// 优先使用编译期配置，其次运行环境变量，最后使用公开客户端。
pub fn ms_oauth_client_id() -> String {
    usable_key(option_env!("MS_CLIENT_ID").map(str::to_owned))
        .or_else(|| usable_key(std::env::var("MS_CLIENT_ID").ok()))
        .or_else(|| usable_key(std::env::var("PCL_MS_CLIENT_ID").ok()))
        .unwrap_or_else(|| "00000000402b5328".to_owned())
}

/// CurseForge 是否可用，供界面隐藏或禁用相关选项（对照 CE 的 SupportCurseForge）。
pub fn curseforge_available() -> bool {
    curseforge_api_key().is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blank_and_missing_keys_are_treated_as_unconfigured() {
        assert_eq!(usable_key(None), None);
        assert_eq!(usable_key(Some("   ".into())), None);
        assert_eq!(
            usable_key(Some("  key-id  ".into())).as_deref(),
            Some("key-id")
        );
    }

    #[test]
    fn microsoft_client_id_has_a_default() {
        if usable_key(option_env!("MS_CLIENT_ID").map(str::to_owned)).is_none()
            && usable_key(std::env::var("MS_CLIENT_ID").ok()).is_none()
            && usable_key(std::env::var("PCL_MS_CLIENT_ID").ok()).is_none()
        {
            assert_eq!(ms_oauth_client_id(), "00000000402b5328");
        }
    }
}
