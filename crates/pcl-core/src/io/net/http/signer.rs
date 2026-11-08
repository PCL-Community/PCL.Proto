//! 传输层凭据：由下载源要求的附加请求头。
//!
//! 核心库不读取也不保存密钥，宿主在构造 [`crate::io::net::http::HttpTransport`] 时注入实现。
//! 是否附带、附带到哪些主机完全由实现决定；镜像源是第三方地址，实现必须只对官方主机签名，
//! 否则密钥会被转发给镜像运营方。

use reqwest::header::HeaderMap;

/// 按目标 URL 生成附加请求头，返回 `None` 表示不需要签名。
pub trait RequestSigner: Send + Sync {
    fn headers(&self, url: &reqwest::Url) -> Option<HeaderMap>;
}

/// CurseForge 的 `x-api-key`：官方 API 与官方 CDN 都按主机校验。
///
/// 对照 CE 的 `RequestSigning.SecretHeadersSign`，只签下列官方主机；镜像源（mcim）
/// 与网页域名都不在此列，密钥不会离开官方域名。
pub struct CurseForgeSigner {
    key: reqwest::header::HeaderValue,
}

impl CurseForgeSigner {
    /// 空白密钥按未配置处理，返回 `None` 时不注册签名器。
    pub fn new(key: &str) -> Option<Self> {
        let key = key.trim();
        if key.is_empty() {
            return None;
        }
        let mut value = reqwest::header::HeaderValue::from_bytes(key.as_bytes()).ok()?;
        value.set_sensitive(true);
        Some(Self { key: value })
    }

    fn is_official_host(host: &str) -> bool {
        matches!(
            host,
            "api.curseforge.com" | "edge.forgecdn.net" | "mediafilez.forgecdn.net"
        ) || host == "cursecdn.com"
            || host.ends_with(".cursecdn.com")
    }
}

impl RequestSigner for CurseForgeSigner {
    fn headers(&self, url: &reqwest::Url) -> Option<HeaderMap> {
        if url.scheme() != "https" || !Self::is_official_host(url.host_str()?) {
            return None;
        }
        Some(HeaderMap::from_iter([(
            reqwest::header::HeaderName::from_static("x-api-key"),
            self.key.clone(),
        )]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signs_only_official_https_hosts() {
        let signer = CurseForgeSigner::new("  secret-key  ").unwrap();
        for signed in [
            "https://api.curseforge.com/v1/mods/1",
            "https://edge.forgecdn.net/files/9453/343/a.jar",
            "https://mediafilez.forgecdn.net/files/9453/343/a.jar",
            "https://cursecdn.com/files/a.jar",
        ] {
            let headers = signer
                .headers(&reqwest::Url::parse(signed).unwrap())
                .unwrap();
            assert_eq!(headers["x-api-key"], "secret-key", "{signed}");
        }
        for unsigned in [
            // 镜像源、网页域名与第三方站点都不能收到官方密钥。
            "https://mod.mcimirror.top/curseforge/v1/mods/1",
            "https://cdn.modrinth.com/data/a.jar",
            "https://www.curseforge.com/minecraft/mc-mods/jei",
            "https://api.curseforge.com.evil.test/v1/mods/1",
            "http://edge.forgecdn.net/files/a.jar",
        ] {
            assert!(
                signer
                    .headers(&reqwest::Url::parse(unsigned).unwrap())
                    .is_none(),
                "{unsigned}"
            );
        }
        assert!(CurseForgeSigner::new("   ").is_none());
        assert!(CurseForgeSigner::new("bad\nkey").is_none());
    }
}
