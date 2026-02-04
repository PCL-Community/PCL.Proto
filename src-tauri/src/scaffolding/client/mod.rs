mod states;
use std::{sync::Arc, time::Duration};

use super::room;
use crate::{
    scaffolding::{
        ScaffoldingError, easytier::EasyTier, server::request_handler::RequestHandler,
        util::room_code::RoomCode,
    },
    setup::ConfigManager,
};

struct ScaffoldingClient {
    room: Option<room::Room>,
    easytier: EasyTier,
    player: room::member::Member,
    server_node_ip: Option<String>,
    protocols: Vec<&'static str>,
    client_state: states::TerracottaState,
}

impl ScaffoldingClient {
    pub fn new(easytier: EasyTier, player_name: &str, vendor: &str) -> Self {
        Self {
            room: None,
            easytier,
            player: room::member::Member {
                name: player_name.to_string(),
                machine_id: ConfigManager::instance().pcl_identifier.clone(),
                vendor: vendor.to_string(),
                kind: room::member::PlayerKind::Guest,
            },
            server_node_ip: None,
            client_state: states::TerracottaState::Idle,
            protocols: RequestHandler::new().protocols(),
        }
    }

    /// 连接到房间。
    ///
    /// 该方法返回后，必须每隔 5s 调用一次 `heartbeat()` 方法。
    /// [标准联机流程](https://github.com/Scaffolding-MC/Scaffolding-MC/blob/main/README.md#拓展协议)
    /// - Parameters:
    ///   - room_code: 房间码。
    ///   - check_server: 是否检查联机中心返回的 Minecraft 服务器端口号。
    pub fn connect(&mut self, room_code: &str, check_server: bool) -> Result<(), ScaffoldingError> {
        let room_code = RoomCode::from_code(room_code).ok_or(ScaffoldingError::InvalidRoomCode)?;
        let room_code = Arc::new(room_code);
        self.client_state = states::TerracottaState::GuestConnecting {
            room: room_code.clone(),
        };
        let config = room_code.compute_arguments_guest(None);
        self.easytier
            .launch(config)
            .map_err(|e| ScaffoldingError::EasyTierError(e.to_string()))?;
        for _ in 0..15 {
            std::thread::sleep(Duration::from_secs(1));
        }
        Ok(())
    }
}
