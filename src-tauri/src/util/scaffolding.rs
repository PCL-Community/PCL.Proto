use easytier::{instance_manager::NetworkInstanceManager, launcher::NetworkConfig};

#[derive(thiserror::Error, Debug)]
enum LinkError {
    #[error("Invalid invite code")]
    InviteCodeFormatMismatch,

    #[error("Invalid port: {0}")]
    InvalidPort(#[from] std::num::ParseIntError),
}

#[derive(Debug)]
enum LinkInvitation {
    PureConnetLabeling {
        port: u16,
        network_name: String,
        network_secret: String,
        extra: Option<String>,
    },
    Scanfolding {
        network_name: String,
        network_secret: String,
    },
}

impl LinkInvitation {
    fn new_with_port(port: u16) -> Self {
        Self::Scanfolding {
            network_name: "".to_string(),
            network_secret: "".to_string(),
        }
    }

    fn from_invite_code(invite_code: &str) -> Result<Self, LinkError> {
        if invite_code.starts_with('P') {
            // Pure Connect Labeling
            if invite_code.len() < 16 {
                return Err(LinkError::InviteCodeFormatMismatch);
            }
            let port_raw = &invite_code[1..5];
            let network_name = &invite_code[0..11];
            let network_secret = &invite_code[12..17];
            let extra_info = invite_code.get(18..); // optional, may be used later
            let port_num = u16::from_str_radix(port_raw, 16)?;
            Ok(Self::PureConnetLabeling {
                port: port_num,
                network_name: network_name.to_string(),
                network_secret: network_secret.to_string(),
                extra: extra_info.map(String::from),
            })
        } else if invite_code.starts_with("U/") {
            if invite_code.len() != 21 {
                return Err(LinkError::InviteCodeFormatMismatch);
            }
            let network_name = &invite_code[2..11];
            let network_secret = &invite_code[12..21];
            Ok(Self::Scanfolding {
                network_name: format!("scaffolding-mc-{}", network_name),
                network_secret: network_secret.to_string(),
            })
        } else {
            Err(LinkError::InviteCodeFormatMismatch)
        }
    }
}

impl Into<NetworkConfig> for LinkInvitation {
    fn into(self) -> NetworkConfig {
        let mut cfg = NetworkConfig::default();
        match self {
            LinkInvitation::PureConnetLabeling {
                port,
                network_name,
                network_secret,
                extra,
            } => {
                cfg.public_server_url = Some("tcp://public2.easytier.cn:54321".to_string());
                cfg.network_name = Some(network_name);
                cfg.network_secret = Some(network_secret);
                // cfg.virtual_ipv4 = Some("10.114.114.114".to_string());
                cfg.hostname = Some("Client-".to_string());
            }
            LinkInvitation::Scanfolding {
                network_name,
                network_secret,
            } => {}
        }
        cfg
    }
}

#[tauri::command]
pub fn start_connection_from_code(
    network_instance_manager: tauri::State<'_, NetworkInstanceManager>,
    code: &str,
) -> Result<uuid::Uuid, String> {
    let invitation = LinkInvitation::from_invite_code(code).map_err(|err| err.to_string())?;
    let config: NetworkConfig = invitation.into();
    let config_loader = config.gen_config().unwrap();
    let instance_id = network_instance_manager
        .run_network_instance(config_loader, easytier::launcher::ConfigSource::FFI)
        .unwrap();
    log::debug!("connected with instance_id: {}", instance_id);
    Ok(instance_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use easytier::launcher::NetworkInstance;

    #[test]
    fn test_parse_code() {
        use std::thread::sleep;
        let invitation1 = LinkInvitation::from_invite_code("P0ABB-017LK-S1DZ1-V2").unwrap();
        dbg!(&invitation1);
        let config: NetworkConfig = invitation1.into();
        let config_toml = config.gen_config().unwrap();
        dbg!(config, &config_toml);
        let mut net = NetworkInstance::new(config_toml, easytier::launcher::ConfigSource::FFI);
        let _subscriber = net.start().unwrap();
        println!("EasyTier started (programmatic).");
        sleep(std::time::Duration::from_mins(1));
        let invitation2 = LinkInvitation::from_invite_code("U/NNNN-NNNN-SSSS-SSSS").unwrap();
        dbg!(&invitation2);
    }

    #[test]
    fn instance_manage() {
        use easytier::common::config::TomlConfigLoader;
        use easytier::launcher::ConfigSource;
        // 创建 manager
        let manager = NetworkInstanceManager::new();
        // 配置
        let cfg_str = r#"
            listeners = ["tcp://0.0.0.0:12345"]
        "#;
        let loader = TomlConfigLoader::new_from_str(cfg_str).unwrap();
        // 启动实例（连接）
        let instance_id = manager
            .run_network_instance(loader, ConfigSource::FFI)
            .unwrap();
        // 关闭实例
        manager.delete_network_instance(vec![instance_id]).unwrap();
    }
}
