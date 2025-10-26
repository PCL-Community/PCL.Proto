//! from PCL.Core/Link/McPing.cs and PCL.Core/Utils/VarIntHelper.cs
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};

struct MCPing {
    endpoint: SocketAddr,
    timeout: std::time::Duration,
    host: String,
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
            let endpoint_raw = if !addr_str.split_once(':').is_none() {
                addr_str
            } else {
                &format!("{}:{}", addr_str, Self::DEFAULT_PORT)
            };
            let socket_addrs: Vec<SocketAddr> =
                tokio::net::lookup_host(endpoint_raw).await?.collect();
            *socket_addrs
                .first()
                .ok_or_else(|| anyhow::anyhow!("cannot parse addr: {}", addr_str))?
        };
        Ok(Self {
            endpoint: socket_addr,
            timeout: Self::DEFAULT_TIMEOUT,
            host: socket_addr.ip().to_string(),
        })
    }

    pub async fn ping(&self) -> anyhow::Result<(serde_json::Value, u128)> {
        let mut socket_stream = tokio::net::TcpStream::connect(&self.endpoint).await?;
        log::debug!("Connection established: {}", self.endpoint);
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
            log::debug!("Received JSON");
            Ok((
                serde_json::from_str::<serde_json::Value>(&json_str)?,
                latency,
            ))
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
    pub fn _decode(bytes: &[u8], read_length: &mut u8) -> Result<usize> {
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
pub async fn server_query(addr_str: &str) -> Result<(serde_json::Value, u128), String> {
    let mc_ping = MCPing::from_str(addr_str)
        .await
        .map_err(|e| e.to_string())?;
    log::debug!("constructed mc_ping with endpoint: {}", mc_ping.endpoint);
    mc_ping.ping().await.map_err(|e| e.to_string())
}

#[cfg(test)]
#[tokio::test]
async fn mc_ping_test() -> anyhow::Result<()> {
    let input = "127.0.0.1";
    let mcping = MCPing::from_str(input).await?;
    dbg!(mcping.endpoint);
    match mcping.ping().await {
        Ok(result) => {
            println!("result json: {}", result.0);
            println!("latency: {}", result.1);
        }
        Err(e) => {
            println!("failed to connet for {}", e)
        }
    }
    Ok(())
    // dbg!(socket_addr.ip());
    // dbg!(socket_addr.port());
}
