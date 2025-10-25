//! from PCL.Core/Link/McPing.cs and PCL.Core/Utils/VarIntHelper.cs
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct MCPing {
    endpoint: SocketAddr,
    timeout: std::time::Duration,
    host: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McPingResult {
    pub version: McPingVersionResult,
    pub players: McPingPlayerResult,
    pub description: String,
    pub favicon: Option<String>,
    pub latency: u128,
    pub modinfo: Option<McPingModInfoResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McPingVersionResult {
    pub name: String,
    pub protocol: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McPingPlayerResult {
    pub max: i32,
    pub online: i32,
    pub sample: Vec<McPingPlayerSampleResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McPingPlayerSampleResult {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McPingModInfoResult {
    pub r#type: String,
    pub mod_list: Vec<McPingModInfoModResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct McPingModInfoModResult {
    pub modid: String,
    pub version: String,
}

impl MCPing {
    const DEFAULT_PORT: u16 = 25565;
    const DEFAULT_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(10000);

    /// parse a str to MCPing endpoint
    pub async fn from_str(addr_str: &str) -> anyhow::Result<Self> {
        let socket_addr = if let Ok(addr) = addr_str.parse::<SocketAddr>() {
            addr
        } else if let Ok(ip) = addr_str.parse::<IpAddr>() {
            SocketAddr::new(ip, Self::DEFAULT_PORT)
        } else {
            let socket_addrs: Vec<SocketAddr> =
                tokio::net::lookup_host(format!("{}:{}", addr_str, Self::DEFAULT_PORT))
                    .await?
                    .collect();
            socket_addrs
                .first()
                .ok_or_else(|| anyhow::anyhow!("cannot parse addr: {}", addr_str))?
                .to_owned()
        };
        Ok(Self {
            endpoint: socket_addr,
            timeout: Self::DEFAULT_TIMEOUT,
            host: socket_addr.ip().to_string(),
        })
    }

    pub async fn ping(&self) -> anyhow::Result<McPingResult> {
        log::debug!("Connecting to {}", self.endpoint);
        let mut socket_stream = tokio::net::TcpStream::connect(&self.endpoint).await?;
        log::debug!("Connection established:{}", self.endpoint);
        let handshake_packet = self.build_handshake_packet();
        tokio::time::timeout(self.timeout, async {
            socket_stream.write_all(&handshake_packet).await?;
            log::debug!("Handshake sent, packet length: {}", handshake_packet.len());

            // 构建并发送状态请求包
            let status_packet = self.build_status_request_packet();
            socket_stream.write_all(&status_packet).await?;
            log::debug!("Status sent, packet length: {}", status_packet.len());

            // 读取响应
            let start = std::time::Instant::now();

            // 读取包长度 (VarInt)
            let total_length = varint::read_from_stream(&mut socket_stream).await?;
            log::debug!("Total length: {}", total_length);

            // 读取包ID (VarInt)
            let packet_id = varint::read_from_stream(&mut socket_stream).await?;
            log::debug!("Packet ID: {}", packet_id);

            // 读取数据长度 (VarInt)
            let data_length = varint::read_from_stream(&mut socket_stream).await?;
            log::debug!("Data length: {}", data_length);

            // 读取JSON数据
            let mut json_buffer = vec![0u8; data_length as usize];
            socket_stream.read_exact(&mut json_buffer).await?;

            let latency = start.elapsed().as_millis();

            let json_str = String::from_utf8(json_buffer)?;
            log::debug!("Received JSON: {}", json_str);

            self.parse_response(&json_str, latency)
        })
        .await?
    }

    /// 构建握手包
    fn build_handshake_packet(&self) -> Vec<u8> {
        let mut handshake = Vec::new();
        handshake.extend_from_slice(&varint::encode(0)); // 状态头 表明这是一个握手包
        handshake.extend_from_slice(&varint::encode(772)); // 协议头 表明请求客户端的版本
        let binary_ip = self.host.as_bytes();
        handshake.extend_from_slice(&varint::encode(binary_ip.len())); //服务器地址长度
        handshake.extend_from_slice(binary_ip); //服务器地址
        handshake.extend_from_slice(&self.endpoint.port().to_be_bytes()); //服务器端口
        handshake.extend_from_slice(&varint::encode(1)); //1 表明当前状态为 ping 2 表明当前的状态为连接
        // 添加包长度前缀
        let mut packet = Vec::new();
        packet.extend_from_slice(&varint::encode(handshake.len()));
        packet.extend_from_slice(&handshake);
        packet
    }

    fn build_status_request_packet(&self) -> Vec<u8> {
        let mut packet = Vec::new();
        // 包长度 (包ID + 数据)
        packet.extend_from_slice(&varint::encode(1)); // 长度 = 1 (只有包ID)
        // 包ID (0 for status request)
        packet.extend_from_slice(&varint::encode(0));
        packet
    }

    /// 解析收取的数据包的JSON内容
    fn parse_response(&self, json_str: &str, latency: u128) -> anyhow::Result<McPingResult> {
        let json: serde_json::Value = serde_json::from_str(json_str)?;

        let version = json
            .get("version")
            .ok_or_else(|| anyhow::anyhow!("Missing version field"))?;

        let players = json.get("players").unwrap_or(&serde_json::Value::Null);
        let description = self.convert_description_to_string(
            json.get("description").unwrap_or(&serde_json::Value::Null),
        );
        let favicon = json
            .get("favicon")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        let modinfo = json.get("modinfo");

        let version_result = McPingVersionResult {
            name: version
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("未知服务端版本名")
                .to_string(),
            protocol: version
                .get("protocol")
                .and_then(|v| v.as_i64())
                .unwrap_or(-1) as i32,
        };

        let players_result = McPingPlayerResult {
            max: players.get("max").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            online: players.get("online").and_then(|v| v.as_i64()).unwrap_or(0) as i32,
            sample: players
                .get("sample")
                .and_then(|v| v.as_array())
                .unwrap_or(&vec![])
                .iter()
                .filter_map(|player| {
                    Some(McPingPlayerSampleResult {
                        name: player.get("name")?.as_str()?.to_string(),
                        id: player.get("id")?.as_str()?.to_string(),
                    })
                })
                .collect(),
        };

        let modinfo_result = if let Some(modinfo) = modinfo {
            Some(McPingModInfoResult {
                r#type: modinfo
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("未知服务端类型")
                    .to_string(),
                mod_list: modinfo
                    .get("modList")
                    .and_then(|v| v.as_array())
                    .unwrap_or(&vec![])
                    .iter()
                    .filter_map(|mod_item| {
                        if mod_item.get("modid").is_some() {
                            Some(McPingModInfoModResult {
                                modid: mod_item.get("modid")?.as_str()?.to_string(),
                                version: mod_item.get("version")?.as_str()?.to_string(),
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
            })
        } else {
            None
        };

        Ok(McPingResult {
            version: version_result,
            players: players_result,
            description,
            favicon,
            latency,
            modinfo: modinfo_result,
        })
    }

    fn convert_description_to_string(&self, description: &serde_json::Value) -> String {
        match description {
            serde_json::Value::String(s) => s.clone(),
            serde_json::Value::Object(obj) => self.parse_description_object(obj),
            serde_json::Value::Array(arr) => arr
                .iter()
                .map(|item| self.convert_description_to_string(item))
                .collect::<Vec<String>>()
                .join(""),
            _ => String::new(),
        }
    }

    fn parse_description_object(&self, obj: &serde_json::Map<String, serde_json::Value>) -> String {
        let mut result = String::new();

        // 处理 extra 数组
        if let Some(extra) = obj.get("extra").and_then(|v| v.as_array()) {
            for item in extra {
                result.push_str(&self.convert_description_to_string(item));
            }
        }

        // 处理 text 字段
        if let Some(text) = obj.get("text").and_then(|v| v.as_str()) {
            let format_code = self.get_text_style_string(
                obj.get("color").and_then(|v| v.as_str()).unwrap_or(""),
                obj.get("bold").and_then(|v| v.as_bool()).unwrap_or(false),
                obj.get("obfuscated")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                obj.get("strikethrough")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                obj.get("underline")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false),
                obj.get("italic").and_then(|v| v.as_bool()).unwrap_or(false),
            );
            result.push_str(&format!("{}{}", format_code, text));
        }
        result
    }

    fn get_text_style_string(
        &self,
        color: &str,
        bold: bool,
        obfuscated: bool,
        strikethrough: bool,
        underline: bool,
        italic: bool,
    ) -> String {
        let color_map = [
            ("black", "0"),
            ("dark_blue", "1"),
            ("dark_green", "2"),
            ("dark_aqua", "3"),
            ("dark_red", "4"),
            ("dark_purple", "5"),
            ("gold", "6"),
            ("gray", "7"),
            ("dark_gray", "8"),
            ("blue", "9"),
            ("green", "a"),
            ("aqua", "b"),
            ("red", "c"),
            ("light_purple", "d"),
            ("yellow", "e"),
            ("white", "f"),
        ];

        let mut result = String::new();

        if let Some(code) =
            color_map.iter().find_map(
                |(name, code)| {
                    if *name == color { Some(*code) } else { None }
                },
            )
        {
            result.push_str(&format!("§{}", code));
        }

        if bold {
            result.push_str("§l");
        }
        if italic {
            result.push_str("§o");
        }
        if underline {
            result.push_str("§n");
        }
        if strikethrough {
            result.push_str("§m");
        }
        // obfuscated 暂时不使用

        result
    }
}

mod varint {
    use anyhow::Result;
    const MAX_BYTES: u8 = 10;

    /// 将无符号长整数编码为VarInt字节序列
    pub fn encode(mut value: usize) -> Vec<u8> {
        let mut result = Vec::with_capacity(MAX_BYTES as usize);
        while value > 0x7F {
            result.push((value as u8) | 0x80);
            value >>= 7;
        }
        result.push(value as u8);
        result
    }

    /// 从字节数组中解码无符号长整数
    pub fn decode(bytes: &[u8], read_length: &mut u8) -> Result<usize> {
        let mut result: usize = 0;
        let mut shift = 0;
        let mut bytes_read = 0;

        for &byte in bytes {
            if bytes_read > MAX_BYTES {
                return Err(anyhow::anyhow!("VarInt exceeds maximum length"));
            }
            result |= ((byte & 0x7F) as usize) << shift;
            bytes_read += 1;
            if (byte & 0x80) == 0 {
                *read_length = bytes_read;
                return Ok(result);
            }
            shift += 7;
        }
        return Err(anyhow::anyhow!("Incomplete VarInt encoding"));
    }

    pub async fn read_from_stream(stream: &mut tokio::net::TcpStream) -> Result<i32> {
        use tokio::io::{AsyncReadExt, ErrorKind};

        let mut result = 0;
        let mut shift = 0;
        let mut buffer = [0u8; 1];

        loop {
            match stream.read_exact(&mut buffer).await {
                Ok(_) => {
                    let byte = buffer[0];
                    result |= ((byte & 0x7F) as i32) << shift;
                    shift += 7;

                    if (byte & 0x80) == 0 {
                        break;
                    }

                    if shift >= 32 {
                        return Err(anyhow::anyhow!("VarInt too large"));
                    }
                }
                Err(e) if e.kind() == ErrorKind::UnexpectedEof => {
                    return Err(anyhow::anyhow!("Unexpected EOF while reading VarInt"));
                }
                Err(e) => return Err(e.into()),
            }
        }

        Ok(result)
    }
}

#[tauri::command]
pub async fn server_query(addr_str: &str) -> Result<McPingResult, String> {
    let mc_ping = MCPing::from_str(addr_str)
        .await
        .map_err(|e| e.to_string())?;
    Ok(mc_ping.ping().await.map_err(|e| e.to_string())?)
}

#[cfg(test)]
#[tokio::test]
async fn mc_ping_test() -> anyhow::Result<()> {
    let input = "127.0.0.1";
    let mcping = MCPing::from_str(input).await?;
    dbg!(mcping.endpoint);
    match mcping.ping().await {
        Ok(result) => {
            println!("服务器版本: {}", result.version.name);
            println!("在线玩家: {}/{}", result.players.online, result.players.max);
            println!("延迟: {}ms", result.latency);
            println!("描述: {}", result.description);
        }
        Err(e) => {
            println!("failed to connet for {}", e)
        }
    }
    Ok(())
    // dbg!(socket_addr.ip());
    // dbg!(socket_addr.port());
}
