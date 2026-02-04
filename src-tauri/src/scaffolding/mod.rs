use serde::Serialize;
pub mod commands;
mod easytier;
pub mod mc;
pub mod protocol;
pub mod terracotta;

/// 联机错误
#[derive(Serialize)]
pub enum ScaffoldingError {
    /// 房间码无效
    InvalidRoomCode,
    /// 联机中心未找到
    HostNotFound,
    /// 协议错误
    ProtocolError(String),
    /// 网络错误
    NetworkError(String),
    /// EasyTier 错误
    EasyTierError(String),
    /// 其他错误
    Other(String),
}