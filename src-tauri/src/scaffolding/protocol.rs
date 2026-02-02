use super::PlayerInfo;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::Arc;
use tokio::sync::Mutex;

/// 协议处理器
pub struct ProtocolHandler {
    /// 玩家列表
    players: Arc<Mutex<Vec<PlayerInfo>>>,
    /// Minecraft服务器端口
    server_port: u16,
}

impl ProtocolHandler {
    /// 创建新的协议处理器
    pub fn new(server_port: u16) -> Self {
        Self {
            players: Arc::new(Mutex::new(Vec::new())),
            server_port,
        }
    }

    /// 启动TCP服务器
    pub async fn start_server(&self, addr: &str) -> Result<(), String> {
        let listener = TcpListener::bind(addr).map_err(|e| e.to_string())?;
        
        println!("Scaffolding protocol server started at {}", addr);
        
        for stream in listener.incoming() {
            match stream {
                Ok(stream) => {
                    let players = self.players.clone();
                    let server_port = self.server_port;
                    
                    tokio::spawn(async move {
                        if let Err(e) = handle_client(stream, players, server_port).await {
                            println!("Error handling client: {}", e);
                        }
                    });
                }
                Err(e) => {
                    println!("Error accepting connection: {}", e);
                }
            }
        }
        
        Ok(())
    }

    /// 发送协议请求
    pub async fn send_request(
        &self,
        addr: &str,
        request_type: &str,
        request_body: &[u8],
    ) -> Result<Vec<u8>, String> {
        let mut stream = TcpStream::connect(addr).map_err(|e| e.to_string())?;
        
        // 发送请求
        let request_type_len = request_type.len() as u8;
        stream.write_all(&[request_type_len]).map_err(|e| e.to_string())?;
        stream.write_all(request_type.as_bytes()).map_err(|e| e.to_string())?;
        
        let request_body_len = request_body.len() as u32;
        stream.write_all(&request_body_len.to_be_bytes()).map_err(|e| e.to_string())?;
        stream.write_all(request_body).map_err(|e| e.to_string())?;
        
        // 读取响应
        let mut status_buf = [0u8; 1];
        stream.read_exact(&mut status_buf).map_err(|e| e.to_string())?;
        
        let mut response_len_buf = [0u8; 4];
        stream.read_exact(&mut response_len_buf).map_err(|e| e.to_string())?;
        let response_len = u32::from_be_bytes(response_len_buf) as usize;
        
        let mut response_body = vec![0u8; response_len];
        stream.read_exact(&mut response_body).map_err(|e| e.to_string())?;
        
        Ok(response_body)
    }
}

/// 处理客户端连接
async fn handle_client(
    mut stream: TcpStream,
    players: Arc<Mutex<Vec<PlayerInfo>>>,
    server_port: u16,
) -> Result<(), String> {
    // 读取请求类型长度
    let mut request_type_len_buf = [0u8; 1];
    stream.read_exact(&mut request_type_len_buf).map_err(|e| e.to_string())?;
    let request_type_len = request_type_len_buf[0] as usize;
    
    // 读取请求类型
    let mut request_type_buf = vec![0u8; request_type_len];
    stream.read_exact(&mut request_type_buf).map_err(|e| e.to_string())?;
    let request_type = String::from_utf8(request_type_buf).map_err(|e| e.to_string())?;
    
    // 读取请求体长度
    let mut request_body_len_buf = [0u8; 4];
    stream.read_exact(&mut request_body_len_buf).map_err(|e| e.to_string())?;
    let request_body_len = u32::from_be_bytes(request_body_len_buf) as usize;
    
    // 读取请求体
    let mut request_body = vec![0u8; request_body_len];
    stream.read_exact(&mut request_body).map_err(|e| e.to_string())?;
    
    // 处理请求
    let (status, response_body) = match request_type.as_str() {
        "c:ping" => handle_ping(&request_body).await,
        "c:protocols" => handle_protocols(&request_body).await,
        "c:server_port" => handle_server_port(server_port).await,
        "c:player_ping" => handle_player_ping(&request_body, &players).await,
        "c:player_profiles_list" => handle_player_profiles_list(&players).await,
        _ => (255, "Unknown protocol".as_bytes().to_vec()),
    };
    
    // 发送响应
    stream.write_all(&[status]).map_err(|e| e.to_string())?;
    stream.write_all(&(response_body.len() as u32).to_be_bytes()).map_err(|e| e.to_string())?;
    stream.write_all(&response_body).map_err(|e| e.to_string())?;
    
    Ok(())
}

/// 处理c:ping请求
async fn handle_ping(request_body: &[u8]) -> (u8, Vec<u8>) {
    // 响应体与请求体相同
    (0, request_body.to_vec())
}

/// 处理c:protocols请求
async fn handle_protocols(request_body: &[u8]) -> (u8, Vec<u8>) {
    // 解析客户端支持的协议列表
    let client_protocols_str = String::from_utf8_lossy(request_body).to_string();
    let client_protocols: Vec<&str> = client_protocols_str
        .split('\0')
        .filter(|s| !s.is_empty())
        .collect();
    
    // 服务器支持的协议列表
    let server_protocols = vec![
        "c:ping",
        "c:protocols",
        "c:server_port",
        "c:player_ping",
        "c:player_profiles_list",
    ];
    
    // 计算交集
    let supported_protocols: Vec<&str> = server_protocols
        .into_iter()
        .filter(|p| client_protocols.contains(&p))
        .collect();
    
    // 构造响应体
    let response_body_str = supported_protocols.join("\0");
    let response_body = response_body_str.as_bytes().to_vec();
    
    (0, response_body)
}

/// 处理c:server_port请求
async fn handle_server_port(server_port: u16) -> (u8, Vec<u8>) {
    // 发送服务器端口（大端序）
    let port_bytes = server_port.to_be_bytes();
    (0, port_bytes.to_vec())
}

/// 处理c:player_ping请求
async fn handle_player_ping(request_body: &[u8], players: &Arc<Mutex<Vec<PlayerInfo>>>) -> (u8, Vec<u8>) {
    // 解析请求体
    if let Ok(player_info) = serde_json::from_slice::<PlayerInfo>(request_body) {
        let mut players_lock = players.lock().await;
        
        // 检查玩家是否已存在
        if let Some(index) = players_lock.iter().position(|p| p.machine_id == player_info.machine_id) {
            // 更新玩家信息
            players_lock[index] = player_info;
        } else {
            // 添加新玩家
            players_lock.push(player_info);
        }
        
        (0, Vec::new())
    } else {
        (255, "Invalid request body".as_bytes().to_vec())
    }
}

/// 处理c:player_profiles_list请求
async fn handle_player_profiles_list(players: &Arc<Mutex<Vec<PlayerInfo>>>) -> (u8, Vec<u8>) {
    let players_lock = players.lock().await;
    
    // 构造响应体
    if let Ok(response_body) = serde_json::to_vec(&*players_lock) {
        (0, response_body)
    } else {
        (255, "Failed to serialize players".as_bytes().to_vec())
    }
}

/// 客户端协议
pub struct ProtocolClient {
    /// 服务器地址
    server_addr: String,
}

impl ProtocolClient {
    /// 创建新的协议客户端
    pub fn new(server_addr: &str) -> Self {
        Self {
            server_addr: server_addr.to_string(),
        }
    }

    /// 发送ping请求
    pub async fn ping(&self, data: &[u8]) -> Result<Vec<u8>, String> {
        let handler = ProtocolHandler::new(0);
        handler.send_request(&self.server_addr, "c:ping", data).await
    }

    /// 协商协议列表
    pub async fn negotiate_protocols(&self, protocols: &[&str]) -> Result<Vec<String>, String> {
        let handler = ProtocolHandler::new(0);
        let request_body_str = protocols.join("\0");
        let request_body = request_body_str.as_bytes();
        
        let response = handler.send_request(&self.server_addr, "c:protocols", request_body).await?;
        let response_str = String::from_utf8(response).map_err(|e| e.to_string())?;
        
        Ok(response_str.split('\0').filter(|s| !s.is_empty()).map(|s| s.to_string()).collect())
    }

    /// 获取服务器端口
    pub async fn get_server_port(&self) -> Result<u16, String> {
        let handler = ProtocolHandler::new(0);
        let response = handler.send_request(&self.server_addr, "c:server_port", &[]).await?;
        
        if response.len() != 2 {
            return Err("Invalid response length".to_string());
        }
        
        let port = u16::from_be_bytes([response[0], response[1]]);
        Ok(port)
    }

    /// 发送玩家心跳
    pub async fn send_player_ping(&self, player_info: &PlayerInfo) -> Result<(), String> {
        let handler = ProtocolHandler::new(0);
        let request_body = serde_json::to_vec(player_info).map_err(|e| e.to_string())?;
        
        let _ = handler.send_request(&self.server_addr, "c:player_ping", &request_body).await?;
        Ok(())
    }

    /// 获取玩家列表
    pub async fn get_player_profiles(&self) -> Result<Vec<PlayerInfo>, String> {
        let handler = ProtocolHandler::new(0);
        let response = handler.send_request(&self.server_addr, "c:player_profiles_list", &[]).await?;
        
        let players = serde_json::from_slice(&response).map_err(|e| e.to_string())?;
        Ok(players)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_handler() {
        let handler = ProtocolHandler::new(25565);
        assert_eq!(handler.server_port, 25565);
    }

    #[test]
    fn test_protocol_client() {
        let client = ProtocolClient::new("127.0.0.1:33768");
        assert_eq!(client.server_addr, "127.0.0.1:33768");
    }
}
