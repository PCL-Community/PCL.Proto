use crate::scaffolding::{
    // easytier::EasyTierManager,
    mc::scanning,
    terracotta::{
        player::PlayerProfile,
        room::{ConnectionDifficulty, RoomCode},
    },
};
use easytier::launcher::NetworkInstance;
use serde::{Deserialize, Serialize};

/// 陶瓦状态
pub enum TerracottaState {
    Idle,
    HostScanning {
        scanner: scanning::MinecraftScanner,
    },
    HostStarting {
        room: RoomCode,
        port: u16,
    },
    HostOk {
        room: RoomCode,
        port: u16,
        easytier: NetworkInstance,
        player_profiles: Vec<PlayerProfile>,
    },
    GuestConnecting {
        room: RoomCode,
    },
    GuestStarting {
        room: RoomCode,
        easytier: NetworkInstance,
        difficulty: ConnectionDifficulty,
    },
    GuestOk {
        room: RoomCode,
        easytier: NetworkInstance,
        // server: FakeServer,
        player_profiles: Vec<PlayerProfile>,
    },
    Exception(ExceptionType),
}

/// 异常类型
#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ExceptionType {
    PingHostFail,
    PingHostRst,
    GuestEasytierCrash,
    HostEasytierCrash,
    PingServerRst,
    ScaffoldingInvalidResponse,
}

/// 联机房间状态管理
// pub struct RoomState {
//     /// 房间码
//     pub room_code: String,
//     /// 网络名称
//     pub network_name: String,
//     /// 网络密钥
//     pub network_secret: String,
//     /// Minecraft服务器端口
//     pub server_port: u16,
//     /// 玩家列表
//     pub players: Vec<PlayerInfo>,
//     /// 当前状态
//     pub state: TerracottaState,
// }

// impl RoomState {
//     /// 创建新的房间状态
//     pub fn new(
//         room_code: &str,
//         network_name: &str,
//         network_secret: &str,
//         server_port: u16,
//     ) -> Self {
//         Self {
//             room_code: room_code.to_string(),
//             network_name: network_name.to_string(),
//             network_secret: network_secret.to_string(),
//             server_port,
//             players: Vec::new(),
//             state: TerracottaState::Idle,
//         }
//     }

//     /// 更新状态
//     pub fn update_state(&mut self, state: TerracottaState) {
//         self.state = state;
//     }

//     /// 设置错误
//     pub fn set_error(&mut self, error: ExceptionType) {
//         self.state = TerracottaState::Exception(error);
//     }

//     /// 添加玩家
//     pub fn add_player(&mut self, player: PlayerInfo) {
//         if let Some(index) = self
//             .players
//             .iter()
//             .position(|p| p.machine_id == player.machine_id)
//         {
//             self.players[index] = player;
//         } else {
//             self.players.push(player);
//         }
//     }

//     /// 移除玩家
//     pub fn remove_player(&mut self, machine_id: &str) {
//         self.players.retain(|p| p.machine_id != machine_id);
//     }
// }

/// 全局状态管理
pub struct ScaffoldingGlobalState {
    // 房间状态
    // room_state: Arc<Mutex<Option<RoomState>>>,
}

// impl ScaffoldingGlobalState {
//     /// 创建新的全局状态
//     pub fn new() -> Self {
//         Self {
//             // room_state: Arc::new(Mutex::new(None)),
//         }
//     }

//     /// 设置房间状态
//     pub async fn set_room_state(&self, state: RoomState) {
//         let mut guard = self.room_state.lock().await;
//         *guard = Some(state);
//     }

//     /// 清除房间状态
//     pub async fn clear_room_state(&self) {
//         let mut guard = self.room_state.lock().await;
//         *guard = None;
//     }
// }
