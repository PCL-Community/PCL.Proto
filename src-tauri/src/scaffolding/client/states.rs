use crate::scaffolding::{
    easytier::{ConnectionDifficulty, EasyTier}, room::member::Member, util::room_code::RoomCode,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 陶瓦状态
/// 其中包含的easytier直接持有NetworkInstance引用
#[derive(Default)]
pub enum TerracottaState {
    #[default]
    Idle,
    HostScanning {
        // scanner: scanning::MinecraftScanner,
    },
    HostStarting {
        room: Arc<RoomCode>,
        port: u16,
    },
    HostOk {
        room: Arc<RoomCode>,
        port: u16,
        easytier: Arc<EasyTier>,
        player_profiles: Vec<Member>,
    },
    GuestConnecting {
        room: Arc<RoomCode>,
    },
    GuestStarting {
        room: Arc<RoomCode>,
        easytier: Arc<EasyTier>,
        difficulty: ConnectionDifficulty,
    },
    GuestOk {
        room: Arc<RoomCode>,
        easytier: Arc<EasyTier>,
        // server: FakeServer,
        player_profiles: Vec<Member>,
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