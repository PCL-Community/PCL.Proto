use serde::{Deserialize, Serialize};

/// 玩家类型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerType {
    Host,
    Guest,
    Local,
}

/// 玩家信息
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlayerProfile {
    /// 玩家名
    pub name: String,
    /// 机器ID
    pub machine_id: String,
    /// 客户端信息
    pub vendor: String,
    /// 玩家类型
    pub kind: PlayerType,
    /// 最后心跳时间（不序列化）
    #[serde(skip)]
    pub last_seen: Option<std::time::SystemTime>,
}
