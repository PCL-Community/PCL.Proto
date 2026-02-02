use super::easytier::EasyTierManager;
use easytier::rpc_service::remote_client::RemoteClientManager;
use std::net::SocketAddr;

/// 联机中心发现器
pub struct HostDiscovery {
    easy_tier_manager: EasyTierManager,
}

impl HostDiscovery {
    /// 创建新的发现器
    pub fn new() -> Self {
        Self {
            easy_tier_manager: EasyTierManager::new(),
        }
    }

    /// 发现联机中心
    pub async fn discover_host(
        &self,
        network_name: &str,
    ) -> Result<SocketAddr, String> {
        // 这里需要使用EasyTier的API获取网络中的节点信息
        // 由于我们没有直接访问EasyTier网络节点的API，这里暂时返回一个模拟的结果
        // 实际实现中，需要遍历网络中的所有节点，查找符合条件的Hostname
        
        // 模拟实现：假设找到一个节点，其Hostname为"scaffolding-mc-server-33768"
        let hostname = "scaffolding-mc-server-33768";
        
        if let Some(port) = EasyTierManager::parse_hostname(hostname) {
            // 模拟IP地址
            let ip = "100.64.0.100".parse().unwrap();
            Ok(SocketAddr::new(ip, port))
        } else {
            Err("Host not found".to_string())
        }
    }

    /// 验证Hostname是否合法
    pub fn validate_hostname(hostname: &str) -> bool {
        if let Some(port) = EasyTierManager::parse_hostname(hostname) {
            port > 1024 && port <= 65535
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_hostname() {
        // 合法的Hostname
        assert!(HostDiscovery::validate_hostname("scaffolding-mc-server-33768"));
        
        // 端口等于1024，不合法
        assert!(!HostDiscovery::validate_hostname("scaffolding-mc-server-1024"));
        
        // 端口小于1024，不合法
        assert!(!HostDiscovery::validate_hostname("scaffolding-mc-server-808"));
        
        // 端口大于65535，不合法
        assert!(!HostDiscovery::validate_hostname("scaffolding-mc-server-77844"));
        
        // 不是数字端口，不合法
        assert!(!HostDiscovery::validate_hostname("scaffolding-mc-server-n8"));
        
        // 格式不正确，不合法
        assert!(!HostDiscovery::validate_hostname("invalid-hostname"));
    }
}
