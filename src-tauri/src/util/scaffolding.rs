use easytier::proto::api::manage::NetworkConfig;

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
                cfg
            }
            LinkInvitation::Scanfolding {
                network_name,
                network_secret,
            } => cfg,
        }
    }
}

#[cfg(test)]
#[test]
fn test_parse_code() {
    let invitation1 = LinkInvitation::from_invite_code("P0ABB-017LK-S1DZ1-V2").unwrap();
    dbg!(&invitation1);
    let config: NetworkConfig = invitation1.into();
    dbg!(config);
    let invitation2 = LinkInvitation::from_invite_code("U/NNNN-NNNN-SSSS-SSSS").unwrap();
    dbg!(&invitation2);
}
