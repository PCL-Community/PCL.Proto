use easytier::{VERSION as EASYTIER_VERSION, utils::find_free_tcp_port};
use easytier::launcher::NetworkConfig;
use std::net::Ipv4Addr;

/// EasyTier网络配置管理
pub struct EasyTierManager {
    /// 网络配置
    config: Option<NetworkConfig>,
}

impl EasyTierManager {
    /// 创建新的EasyTier管理器
    pub fn new() -> Self {
        Self {
            config: None,
        }
    }

    /// 生成Scaffolding网络配置
    pub fn generate_network_config(
        &mut self,
        network_name: &str,
        network_secret: &str,
        hostname: Option<&str>,
    ) -> Result<NetworkConfig, String> {
        let mut config = NetworkConfig::default();
        
        // 设置默认配置
        // 注意：由于NetworkConfig的方法名可能不同，这里暂时返回默认配置
        // 实际实现中需要根据NetworkConfig的实际方法来设置参数
        
        self.config = Some(config.clone());
        Ok(config)
    }

    /// 获取当前网络配置
    pub fn current_config(&self) -> Option<&NetworkConfig> {
        self.config.as_ref()
    }

    /// 生成Hostname
    pub fn generate_hostname(port: u16) -> String {
        format!("scaffolding-mc-server-{}", port)
    }

    /// 解析Hostname获取端口
    pub fn parse_hostname(hostname: &str) -> Option<u16> {
        if hostname.starts_with("scaffolding-mc-server-") {
            let port_str = hostname.trim_start_matches("scaffolding-mc-server-");
            port_str.parse().ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_free_tcp_port() {
        dbg!(EASYTIER_VERSION);
        let port = find_free_tcp_port(1024..65535).unwrap();
        dbg!(port);
        assert!(port > 0);
    }

    #[test]
    fn test_generate_hostname() {
        let hostname = EasyTierManager::generate_hostname(33768);
        assert_eq!(hostname, "scaffolding-mc-server-33768");
    }

    #[test]
    fn test_parse_hostname() {
        let port = EasyTierManager::parse_hostname("scaffolding-mc-server-33768");
        assert_eq!(port, Some(33768));
        
        let port = EasyTierManager::parse_hostname("invalid-hostname");
        assert_eq!(port, None);
    }
}
