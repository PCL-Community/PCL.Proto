use serde::{Deserialize, Serialize};

/// 玩家类型
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum PlayerKind {
    Host,
    Guest,
}

/// 玩家信息
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct Member {
    /// 玩家名
    pub name: String,
    /// 机器ID
    pub machine_id: String,
    /// 客户端信息
    pub vendor: String,
    /// 玩家类型
    pub kind: PlayerKind,
}
