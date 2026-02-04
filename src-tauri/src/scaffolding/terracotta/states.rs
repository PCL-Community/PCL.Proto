use crate::scaffolding::{
    easytier::ConnectionDifficulty,
    mc::scanning,
    terracotta::{player::PlayerProfile, room::RoomCode},
};
use easytier::launcher::NetworkInstance;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// 陶瓦状态
#[derive(Default)]
pub enum TerracottaState {
    #[default]
    Idle,
    HostScanning {
        scanner: scanning::MinecraftScanner,
    },
    HostStarting {
        room: Arc<RoomCode>,
        port: u16,
    },
    HostOk {
        room: Arc<RoomCode>,
        port: u16,
        easytier: Arc<NetworkInstance>,
        player_profiles: Vec<PlayerProfile>,
    },
    GuestConnecting {
        room: Arc<RoomCode>,
    },
    GuestStarting {
        room: Arc<RoomCode>,
        easytier: Arc<NetworkInstance>,
        difficulty: ConnectionDifficulty,
    },
    GuestOk {
        room: Arc<RoomCode>,
        easytier: Arc<NetworkInstance>,
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

impl TerracottaState {
    pub fn try_shutdown_current(&self) -> anyhow::Result<Arc<RoomCode>> {
        match self {
            TerracottaState::HostOk { room, easytier, .. }
            | TerracottaState::GuestOk { room, easytier, .. } => {
                if let Some(msg) = easytier.get_latest_error_msg() {
                    return Err(anyhow::anyhow!(
                        "EasyTier has encountered an fatal error: {}",
                        msg
                    ));
                }
                let Some(stop_notifier) = easytier.get_stop_notifier().take() else {
                    log::warn!("No stop notifier found for room: {}", room.code);
                    return Err(anyhow::anyhow!(
                        "No stop notifier found for room: {}",
                        room.code
                    ));
                };
                stop_notifier.notify_one();
                Ok(room.clone())
            }
            _ => Err(anyhow::anyhow!("No active room to stop")),
        }
    }
}
