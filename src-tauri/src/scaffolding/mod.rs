pub mod commands;
mod easytier;
pub mod mc;
mod protocol;
pub mod terracotta;

// /// 联机错误
// pub enum ScaffoldingError {
//     /// 房间码无效
//     InvalidRoomCode,
//     /// 联机中心未找到
//     HostNotFound,
//     /// 协议错误
//     ProtocolError(String),
//     /// 网络错误
//     NetworkError(String),
//     /// 其他错误
//     Other(String),
// }

// impl From<ScaffoldingError> for String {
//     fn from(err: ScaffoldingError) -> Self {
//         match err {
//             ScaffoldingError::InvalidRoomCode => "Invalid room code".to_string(),
//             ScaffoldingError::HostNotFound => "Host not found".to_string(),
//             ScaffoldingError::ProtocolError(msg) => format!("Protocol error: {}", msg),
//             ScaffoldingError::NetworkError(msg) => format!("Network error: {}", msg),
//             ScaffoldingError::Other(msg) => msg,
//         }
//     }
// }
