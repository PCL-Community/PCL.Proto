#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProxyMode {
    #[default]
    Off,
    System,
    Custom,
}

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct ProxySettings {
    pub mode: ProxyMode,
    /// Custom proxy URL, possibly carrying credentials (`http://user:pass@host:port`).
    pub address: String,
    /// 自定义代理凭据；地址里已带用户名时以这里为准。
    pub username: String,
    pub password: String,
}

impl ProxySettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.address.len() > 2048
            || self.address.contains('\0')
            || self.username.len() > 512
            || self.password.len() > 512
            || self.username.contains('\0')
            || self.password.contains('\0')
        {
            return Err("Invalid proxy address".into());
        }
        if self.mode != ProxyMode::Custom {
            return Ok(());
        }
        let mut url = reqwest::Url::parse(self.address.trim())
            .map_err(|_| "Invalid proxy address; expected http://host:port".to_string())?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || url.port().is_none()
        {
            return Err("Invalid proxy address; expected http://host:port".into());
        }
        if !self.username.is_empty() {
            url.set_username(&self.username)
                .map_err(|_| "Invalid proxy username".to_string())?;
            url.set_password(Some(&self.password))
                .map_err(|_| "Invalid proxy password".to_string())?;
        }
        Ok(())
    }

    /// Host and port passed to the game JVM (CE only forwards the custom proxy).
    pub fn game_endpoint(&self) -> Option<(String, u16)> {
        if self.mode != ProxyMode::Custom {
            return None;
        }
        let url = reqwest::Url::parse(self.address.trim()).ok()?;
        Some((url.host_str()?.to_owned(), url.port()?))
    }

    /// 自定义代理的完整 URL（含凭据），供 reqwest 使用。
    fn proxy_url(&self) -> Option<reqwest::Url> {
        let mut url = reqwest::Url::parse(self.address.trim()).ok()?;
        if self.username.is_empty() {
            return Some(url);
        }
        // `set_username` 只对特殊 scheme 失败，validate 已限制为 http/https。
        let _ = url.set_username(&self.username);
        let _ = url.set_password(Some(&self.password));
        Some(url)
    }
}


pub fn builder_for(settings: &ProxySettings) -> reqwest::ClientBuilder {
    let builder =
        reqwest::Client::builder().user_agent(crate::config::metadata::constants::USER_AGENT);
    match settings.mode {
        ProxyMode::Off => builder.no_proxy(),
        // reqwest already follows the system proxy configuration by default.
        ProxyMode::System => builder,
        ProxyMode::Custom => {
            match settings.proxy_url().and_then(|url| reqwest::Proxy::all(url).ok()) {
                Some(proxy) => builder.proxy(proxy),
                // Validation rejects malformed addresses, so this only guards stale settings.
                None => builder.no_proxy(),
            }
        }
    }
}

pub fn clients(settings: &ProxySettings) -> Result<(reqwest::Client, reqwest::Client), String> {
    let build = |http1| {
        let mut builder = builder_for(settings)
            .connect_timeout(std::time::Duration::from_secs(15))
            .read_timeout(std::time::Duration::from_secs(15))
            .pool_max_idle_per_host(64)
            .pool_idle_timeout(std::time::Duration::from_secs(90));
        if http1 {
            builder = builder.http1_only();
        }
        builder.build().map_err(|error| error.to_string())
    };
    settings.validate()?;
    Ok((build(false)?, build(true)?))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_custom_addresses_and_reports_game_endpoint() {
        let mut settings = ProxySettings {
            mode: ProxyMode::Custom,
            address: "http://user:secret@127.0.0.1:7890".into(),
        };
        settings.validate().unwrap();
        assert_eq!(settings.game_endpoint(), Some(("127.0.0.1".into(), 7890)));
        settings.address = "127.0.0.1:7890".into();
        assert!(settings.validate().is_err());
        settings.address = "socks5://127.0.0.1:7890".into();
        assert!(settings.validate().is_err());
        settings.mode = ProxyMode::Off;
        assert!(settings.validate().is_ok());
        assert_eq!(settings.game_endpoint(), None);
    }
}
