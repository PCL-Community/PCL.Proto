mod room;
mod states;

use super::PlayerType;

/// 一名参与联机的玩家
struct Member {
    /// 玩家名
    name: String,
    /// 玩家的机器ID
    machine_id: String,
    /// 玩家的联机客户端信息
    vendor: String,
    /// 玩家类型
    kind: PlayerType,
}

/// 一个联机房间
struct RoomState {
    /// 房间中的玩家
    members: Vec<Member>,
    /// Minecraft 服务器端口
    server_port: u16,
}
