//! 读取本应用的 CurseForge / 微软 OAuth 凭据，供具体功能调用使用。
//!
//! 核心库不决定密钥来源：发布构建在编译期把值烧进二进制（对照 CE 的 Source Generator），
//! 开发构建回退到运行环境变量；第三方启动器可以完全换一套读取方式。

use crate::config::manager::ConfigManager;
use std::sync::LazyLock;

pub static CURSEFORGE_API_KEY: LazyLock<Option<String>> = LazyLock::new(|| {
    compile_time(option_env!("CURSEFORGE_API_KEY"))
        .or_else(|| from_env("CURSEFORGE_API_KEY"))
        .or_else(|| from_env("PCL_CURSEFORGE_API_KEY"))
});

/// 空白值按未配置处理。
fn usable(value: Option<String>) -> Option<String> {
    let value = value?.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn compile_time(value: Option<&'static str>) -> Option<String> {
    usable(value.map(str::to_owned))
}

fn from_env(name: &str) -> Option<String> {
    usable(std::env::var(name).ok())
}

pub fn microsoft_provider() -> pcl_core::minecraft::profile::auth::AuthResult<
    pcl_core::minecraft::profile::auth::microsoft::MicrosoftProvider,
> {
    use pcl_core::minecraft::profile::auth::{AuthError, microsoft::MicrosoftProvider};
    static CLIENT_ID: LazyLock<Option<String>> = LazyLock::new(|| {
        compile_time(option_env!("MS_CLIENT_ID"))
            .or_else(|| from_env("MS_CLIENT_ID"))
            .or_else(|| from_env("PCL_MS_CLIENT_ID"))
    });
    let id = CLIENT_ID.as_deref().ok_or_else(|| AuthError::configuration(
        "未配置微软 OAuth 的 Client ID：请在构建时设置 MS_CLIENT_ID，或通过 .env / 运行环境变量提供"
    ))?;
    MicrosoftProvider::new(ConfigManager::instance().api_client.http_client(), id)
}

/// PCL 在皮肤站注册的客户端身份，由宿主选择后交给协议客户端。
pub fn connect_client_id(explicit: Option<&str>, server: &str) -> Option<String> {
    explicit
        .map(str::trim)
        .filter(|id| !id.is_empty())
        .map(str::to_owned)
        .or_else(|| {
            reqwest::Url::parse(server).ok().and_then(|url| {
                url.host_str()
                    .and_then(built_in_client_id)
                    .map(str::to_owned)
            })
        })
}

/// 联机公告服务器根地址（可配置多条，用于多服务器回退）；未配置时联机离线可用。
pub fn link_server_roots() -> Vec<String> {
    compile_time(option_env!("LINK_SERVER_ROOT"))
        .or_else(|| from_env("LINK_SERVER_ROOT"))
        .or_else(|| from_env("PCL_LINK_SERVER_ROOT"))
        .map(|roots| {
            roots
                .split('|')
                .map(str::trim)
                .filter(|root| !root.is_empty())
                .map(|root| root.trim_end_matches('/').to_owned())
                .collect()
        })
        .unwrap_or_default()
}

/// Natayark ID 的 OAuth 客户端凭据；未配置时联机页隐藏账户入口。
pub fn naid_credentials() -> Option<(String, String)> {
    let client_id = compile_time(option_env!("NAID_CLIENT_ID"))
        .or_else(|| from_env("NAID_CLIENT_ID"))
        .or_else(|| from_env("PCL_NAID_CLIENT_ID"))?;
    let client_secret = compile_time(option_env!("NAID_CLIENT_SECRET"))
        .or_else(|| from_env("NAID_CLIENT_SECRET"))
        .or_else(|| from_env("PCL_NAID_CLIENT_SECRET"))?;
    Some((client_id, client_secret))
}

pub fn built_in_client_id(host: &str) -> Option<&'static str> {
    host.trim()
        .trim_end_matches('.')
        .eq_ignore_ascii_case("littleskin.cn")
        .then_some("1514")
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
}
