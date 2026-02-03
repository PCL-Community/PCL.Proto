use std::default;

use crate::scaffolding::{
    // easytier::EasyTierManager,
    easytier::ConnectionDifficulty, mc::scanning, terracotta::{
        player::PlayerProfile,
        room::RoomCode,
    }
};
use easytier::launcher::NetworkInstance;
use serde::{Deserialize, Serialize};

/// 陶瓦状态
#[derive(Default)]
pub enum TerracottaState {
    #[default]
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