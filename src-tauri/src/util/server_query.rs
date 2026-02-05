//! from PCL.Core/Link/McPing.cs and PCL.Core/Utils/VarIntHelper.cs
use super::byte_buffer::ByteBuffer;
use std::net::{IpAddr, SocketAddr};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use trust_dns_resolver::{
    TokioAsyncResolver,
    config::{ResolverConfig, ResolverOpts},
};

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
            let add_str_split = addr_str.split_once(':');
            let (host, port) = match add_str_split {
                Some(split) => (split.0, split.1.parse()?),
                None => (addr_str, Self::DEFAULT_PORT),
            };
            let resolver =
                TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());
            let mut addresses = Vec::new();
            if let Ok(ip_lookup) = resolver.lookup_ip(host).await {
                let ip = ip_lookup.iter().collect::<Vec<_>>();
                addresses.extend_from_slice(&ip);
            } else {
                log::warn!("No A/AAAA record found, falling back to SRV record");
                let srv_name = format!("_minecraft._tcp.{}", host);
                let srv_response = resolver.srv_lookup(&srv_name).await?;
                for srv in srv_response.iter() {
                    let target = srv.target().to_string().trim_end_matches('.').to_string();
                    let srv_port = srv.port();
                    log::debug!("SRV record found: {}:{}", target, srv_port);
                    // 解析SRV记录中的目标主机
                    if let Ok(ip_lookup) = resolver.lookup_ip(&target).await {
                        for ip in ip_lookup.iter() {
                            addresses.push(ip);
                            log::debug!("Resolved SRV target: {}", ip);
                        }
                    }
                }
            }
            if addresses.is_empty() {
                return Err(anyhow::anyhow!("No addresses resolved"));
            }
            let ip = addresses.first().unwrap();
            SocketAddr::new(*ip, port)
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

            // 先读取包长度 (VarInt)
            let total_length = read_varint_from_stream(&mut socket_stream).await?;
            log::debug!("Total length: {}", total_length);

            // 读取剩余数据
            let mut buffer = vec![0u8; total_length as usize];
            socket_stream.read_exact(&mut buffer).await?;

            // 使用 ByteBuffer 解析数据
            let mut byte_buffer = ByteBuffer::new(buffer);
            // 读取包ID (VarInt)
            let packet_id = byte_buffer.read_varint()?;
            log::debug!("Packet ID: {}", packet_id);
            // 读取数据长度 (VarInt)
            let data_length = byte_buffer.read_varint()?;
            log::debug!("Data length: {}", data_length);
            // 读取JSON数据
            let json_buffer = byte_buffer.read_data(data_length as usize)?;
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
        let mut handshake = ByteBuffer::new_empty();
        handshake.write_varint(0); // 状态头 表明这是一个握手包
        handshake.write_varint(772); // 协议头 表明请求客户端的版本
        let binary_ip = self.host.as_bytes();
        handshake.write_varint(binary_ip.len()); //服务器地址长度
        handshake.write_data(binary_ip); //服务器地址
        handshake.write_u16(self.endpoint.port()); //服务器端口
        handshake.write_varint(1); //1 表明当前状态为 ping 2 表明当前的状态为连接
        // 添加包长度前缀
        let mut packet = ByteBuffer::new_empty();
        packet.write_varint(handshake.data.len());
        packet.write_data(&handshake.data);
        packet.data
    }

    fn build_status_request_packet(&self) -> Vec<u8> {
        let mut packet = ByteBuffer::new_empty();
        // 包长度 (包ID + 数据)
        packet.write_varint(1); // 长度 = 1 (只有包ID)
        // 包ID (0 for status request)
        packet.write_varint(0);
        packet.data
    }
}

/// 从 TCP 流中读取 VarInt。
async fn read_varint_from_stream(stream: &mut tokio::net::TcpStream) -> anyhow::Result<i32> {
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
