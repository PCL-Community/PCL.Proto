mod commands;
mod discovery;
mod easytier;
mod protocol;
pub mod terracotta;

use serde::{Deserialize, Serialize};

/// 玩家类型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerType {
    Host,
    Guest,
}

/// 玩家信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerInfo {
    /// 玩家名
    pub name: String,
    /// 机器ID
    pub machine_id: String,
    /// EasyTier节点ID
    pub easytier_id: Option<String>,
    /// 客户端信息
    pub vendor: String,
    /// 玩家类型
    pub kind: PlayerType,
}

/// 联机房间信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RoomInfo {
    /// 房间码
    pub code: String,
    /// 网络名称
    pub network_name: String,
    /// 网络密钥
    pub network_secret: String,
    /// Minecraft服务器端口
    pub server_port: u16,
}

/// 联机错误
pub enum ScaffoldingError {
    /// 房间码无效
    InvalidRoomCode,
    /// 联机中心未找到
    HostNotFound,
    /// 协议错误
    ProtocolError(String),
    /// 网络错误
    NetworkError(String),
    /// 其他错误
    Other(String),
}

impl From<ScaffoldingError> for String {
    fn from(err: ScaffoldingError) -> Self {
        match err {
            ScaffoldingError::InvalidRoomCode => "Invalid room code".to_string(),
            ScaffoldingError::HostNotFound => "Host not found".to_string(),
            ScaffoldingError::ProtocolError(msg) => format!("Protocol error: {}", msg),
            ScaffoldingError::NetworkError(msg) => format!("Network error: {}", msg),
            ScaffoldingError::Other(msg) => msg,
        }
    }
}

pub use commands::*;