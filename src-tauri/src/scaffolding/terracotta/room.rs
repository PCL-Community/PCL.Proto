use easytier::{
    common::config::{ConfigLoader, TomlConfigLoader},
    utils::find_free_tcp_port,
};
/// 参考自 陶瓦联机: `src/controller/rooms/experimental/room.rs`
/// 规范文档：[Scaffolding-MC/Scaffolding-MC](https://github.com/Scaffolding-MC/Scaffolding-MC/blob/main/README.md#联机房间码)
#[derive(Clone)]
pub struct RoomCode {
    /// 房间码
    pub code: String,
    /// 网络名称
    pub network_name: String,
    /// 网络密钥
    pub network_secret: String,
    /// 房间码种子 唯一对应一个房间码
    pub seed: u128,
}

/// 连接难度，根据EasyTier确定的网络结构转换而来
pub enum ConnectionDifficulty {
    Unknown,
    Easiest,
    Simple,
    Medium,
    Tough,
}

/// scaffolding-mc 服务器主机名前缀
static SERVER_PREFIX: &str = "scaffolding-mc-server-";

/// 传入端口 生成Hostname
pub fn generate_hostname(port: u16) -> String {
    format!("{}{}", SERVER_PREFIX, port)
}

/// 解析Hostname 获取端口 格式错误时返回None
pub fn parse_hostname(hostname: &str) -> Option<u16> {
    if hostname.starts_with(SERVER_PREFIX) {
        let port_str = hostname.trim_start_matches(SERVER_PREFIX);
        let port: u16 = port_str.parse().ok()?;
        if port > 1024 { Some(port) } else { None }
    } else {
        None
    }
}

/// 房间码字符集
static ROOM_CODE_CHARSET: &[u8] = "0123456789ABCDEFGHJKLMNPQRSTUVWXYZ".as_bytes();
/// 房间码字符集基数
static BASE_VAL: u128 = 34;

/// 公开服务器列表
pub static PUBLIC_SERVERS: &[&str] = &[
    "tcp://public.easytier.top:11010",
    "tcp://public2.easytier.cn:54321",
    "https://etnode.zkitefly.eu.org/node1",
    "https://etnode.zkitefly.eu.org/node2",
];

impl RoomCode {
    /// 生成符合 Scaffolding-MC 规范的联机房间码
    pub fn generate() -> Self {
        let mut seed = {
            let mut bytes = [0u8; 16];
            getrandom::getrandom(&mut bytes).unwrap();
            u128::from_be_bytes(bytes)
        } % BASE_VAL.pow(16);
        seed -= seed % 7;
        Self::from_seed(seed)
    }

    /// 按照 0-9、A-H、J-N、P-Z顺序映射至 [0, 33] 后，依“小端序”读得的整型应能被7整除
    pub fn from_code(code: &str) -> Option<Self> {
        let code: Vec<char> = code.to_ascii_uppercase().chars().collect();
        if code.len() < "U/XXXX-XXXX-XXXX-XXXX".len() {
            return None;
        }
        let value: u128 = 'value: {
            'parse_segment: for code in code.windows("U/XXXX-XXXX-XXXX-XXXX".len()) {
                if code[0] != 'U' || code[1] != '/' {
                    continue 'parse_segment;
                }

                let code = &code[2..];
                let mut value: u128 = 0;
                for i in (0.."XXXX-XXXX-XXXX-XXXX".len()).rev() {
                    if i == 4 || i == 9 || i == 14 {
                        if code[i] != '-' {
                            continue 'parse_segment;
                        }
                    } else {
                        match Self::lookup_char(code[i]) {
                            Some(v) => value = value * BASE_VAL + v as u128,
                            None => continue 'parse_segment,
                        }
                    }
                }
                if value.is_multiple_of(7) {
                    break 'value value;
                }
            }
            return None;
        };
        Some(Self::from_seed(value))
    }

    /// 从整型值构造房间码
    fn from_seed(mut value: u128) -> Self {
        let mut code = String::with_capacity("U/XXXX-XXXX-XXXX-XXXX".len());
        code.push_str("U/");
        let mut network_name = String::with_capacity("scaffolding-mc-XXXX-XXXX".len());
        network_name.push_str("scaffolding-mc-");
        let mut network_secret = String::with_capacity("XXXX-XXXX".len());

        for i in 0..16 {
            let v = ROOM_CODE_CHARSET[(value % BASE_VAL) as usize] as char;
            value /= BASE_VAL;

            if i == 4 || i == 8 || i == 12 {
                code.push('-');
            }
            code.push(v);

            if i < 8 {
                if i == 4 {
                    network_name.push('-');
                }
                network_name.push(v);
            } else {
                if i == 12 {
                    network_secret.push('-');
                }
                network_secret.push(v);
            }
        }
        Self {
            code,
            network_name,
            network_secret,
            seed: value,
        }
    }

    /// 从字符映射至 [0, 33]
    fn lookup_char(char: char) -> Option<u8> {
        let char = match char {
            'I' => '1',
            'O' => '0',
            _ => char,
        };
        for (j, c) in ROOM_CODE_CHARSET.iter().enumerate() {
            if *c as char == char {
                return Some(j as u8);
            }
        }
        None
    }

    /// 加入房间作为访客
    fn start_room_guest(&self, player: Option<&str>) {}

    /// 加入房间作为主机
    fn start_room_host(&self, port: u16, player: Option<&str>, public_servers: &[&str]) {
        let scaffolding_port = find_free_tcp_port(1024..65535).unwrap();
        let hostname = generate_hostname(scaffolding_port);
        let ipv4 = std::net::Ipv4Addr::new(10, 144, 144, 1);
        // [TODO] 根据上述配置开启一个EasyTier线程
        let network_config = {
            let config = TomlConfigLoader::default();
            config.set_id(uuid::Uuid::new_v4());
            config.set_hostname(Some(hostname));
            config.set_ipv4(Some(ipv4.into()));
            config.set_tcp_whitelist(vec![scaffolding_port.to_string(), port.to_string()]);
            config.set_udp_whitelist(vec![port.to_string()]);
            config
        };
        // 更新状态 这个应该放到commands里去做
        // let hostOk = TerracottaState::HostOk {
        //     room: self.clone(),
        //     port,
        //     easytier: easytier_manager,
        //     player_profiles: Vec::new(),
        // };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_room_code_generate() {
        let room_code = RoomCode::generate();
        dbg!("{}", &room_code.code);
        assert!(RoomCode::from_code(&room_code.code).is_some());
    }

    #[test]
    fn test_room_code_is_valid() {
        let charset_len = ROOM_CODE_CHARSET.len() as u128;
        assert_eq!(charset_len, BASE_VAL);
        assert!(RoomCode::from_code("U/LX2M-2A87-YXMZ-2HJJ").is_some());
        assert!(RoomCode::from_code("U/YS3D-LTH4-6AUC-MBFB").is_some());
        assert!(RoomCode::from_code("U/BC4D-A51Z-ZE3P-LAP9").is_some());
        assert!(RoomCode::from_code("U/UCAW-UY61-QD93-UL9X").is_some());
        assert!(RoomCode::from_code("U/KP8W-WH0Y-NQCY-MXJ4").is_some());
    }

    #[test]
    fn test_generate_hostname() {
        let hostname = generate_hostname(33768);
        assert_eq!(hostname, "scaffolding-mc-server-33768");
    }

    #[test]
    fn test_parse_hostname() {
        let port = parse_hostname("scaffolding-mc-server-33768");
        assert_eq!(port, Some(33768));

        let port = parse_hostname("invalid-hostname");
        assert_eq!(port, None);
    }
}
