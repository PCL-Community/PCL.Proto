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
}

impl ProxySettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.address.len() > 2048 || self.address.contains('\0') {
            return Err("Invalid proxy address".into());
        }
        if self.mode != ProxyMode::Custom {
            return Ok(());
        }
        let url = reqwest::Url::parse(self.address.trim())
            .map_err(|_| "Invalid proxy address; expected http://host:port".to_string())?;
        if !matches!(url.scheme(), "http" | "https")
            || url.host_str().is_none()
            || url.port().is_none()
        {
            return Err("Invalid proxy address; expected http://host:port".into());
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
}

pub fn builder_for(settings: &ProxySettings) -> reqwest::ClientBuilder {
    let builder =
        reqwest::Client::builder().user_agent(crate::config::metadata::constants::USER_AGENT);
    match settings.mode {
        ProxyMode::Off => builder.no_proxy(),
        // reqwest already follows the system proxy configuration by default.
        ProxyMode::System => builder,
        ProxyMode::Custom => match reqwest::Proxy::all(settings.address.trim()) {
            Ok(proxy) => builder.proxy(proxy),
            // Validation rejects malformed addresses, so this only guards stale settings.
            Err(_) => builder.no_proxy(),
        },
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
