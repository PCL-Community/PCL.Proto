use super::terracotta::room::RoomCode;
use crate::scaffolding::ScaffoldingError;
use crate::scaffolding::terracotta::room::PUBLIC_SERVERS;
use crate::scaffolding::terracotta::states::TerracottaState;
use easytier::common::config::ConfigFileControl;
use easytier::launcher::NetworkInstance;
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

/// 启动联机中心
#[tauri::command]
pub async fn start_host(
    terracotta_state: State<'_, Arc<Mutex<TerracottaState>>>,
    player_name: &str,
    port: u16,
) -> Result<String, ScaffoldingError> {
    use crate::scaffolding::protocol::ProtocolHandler;
    use crate::scaffolding::terracotta::player::{PlayerProfile, PlayerType};
    use uuid::Uuid;

    log::info!("Starting host with player: {}, port: {}", player_name, port);
    let room_code = Arc::new(RoomCode::generate());
    log::info!("Generated room code: {}", room_code.code);
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::HostStarting {
            room: room_code.clone(),
            port,
        };
        drop(state);
    }
    log::info!(
        "Attempting to start room host with public servers: {:?}",
        PUBLIC_SERVERS
    );
    let (config, scaffolding_port) = room_code.compute_arguments_host(port, PUBLIC_SERVERS);
    // 直接创建 NetworkInstance
    let mut instance = NetworkInstance::new(config, ConfigFileControl::STATIC_CONFIG);
    instance.start().map_err(|e| {
        log::error!("Failed to start EasyTier instance: {}", e);
        ScaffoldingError::EasyTierError(e.to_string())
    })?;
    let instance = Arc::new(instance);
    
    // 等待 API 服务就绪（参考 Terracotta 实现）
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let mut api_service = None;
    for _ in 0..20 {
        if let Some(service) = instance.get_api_service() {
            api_service = Some(service);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    if api_service.is_none() {
        log::error!("Failed to get API service after retries");
        if let Some(notifier) = instance.get_stop_notifier() {
            notifier.notify_one();
        }
        return Err(ScaffoldingError::EasyTierError("API service not available".to_string()));
    }
    log::info!("EasyTier instance started and API service ready");
    
    // 启动 scaffolding 协议服务器
    let handler = ProtocolHandler::new(port);
    let handler_clone = handler.clone();
    let server_addr = format!("0.0.0.0:{}", scaffolding_port);
    let server_addr_clone = server_addr.clone();
    tokio::spawn(async move {
        if let Err(e) = handler_clone.start_server(&server_addr_clone).await {
            log::error!("Failed to start scaffolding server: {}", e);
        }
    });
    log::info!("Scaffolding server started on {}", server_addr);
    
    // 启动玩家清理任务
    let handler_cleanup = handler.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
        loop {
            interval.tick().await;
            handler_cleanup.cleanup_timeout_players().await;
        }
    });
    
    // 添加房主到玩家列表
    let machine_id = Uuid::new_v4().to_string(); // 临时生成机器ID，实际应根据硬件生成
    let host_player = PlayerProfile {
        name: player_name.to_string(),
        machine_id,
        vendor: "PCL.Proto".to_string(),
        kind: PlayerType::Host,
        last_seen: None,
    };
    handler.add_player(host_player.clone()).await;
    
    // TODO: 启动监控线程
    // 暂时省略监控线程，后续补充
    
    // 更新状态
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::HostOk {
            room: room_code.clone(),
            port,
            easytier: instance,
            player_profiles: vec![host_player],
        };
    }
    log::info!(
        "Room host started successfully with code: {}",
        room_code.code
    );
    Ok(room_code.code.clone())
}

/// 加入联机
#[tauri::command]
pub async fn start_guest(
    terracotta_state: State<'_, Arc<tokio::sync::Mutex<TerracottaState>>>,
    code: &str,
    player_name: &str,
) -> Result<(), ScaffoldingError> {
    use crate::scaffolding::easytier::{ConnectionDifficulty, EasyTierControl, EasyTierMember};
    use crate::scaffolding::protocol::ProtocolClient;
    use crate::scaffolding::terracotta::player::{PlayerProfile, PlayerType};
    use crate::scaffolding::terracotta::room::parse_hostname;
    use uuid::Uuid;

    log::info!(
        "Starting guest with room code: {}, player: {}",
        code,
        player_name
    );
    // 解析房间码
    let room_code = RoomCode::from_code(code).ok_or(ScaffoldingError::InvalidRoomCode)?;
    let room_code = Arc::new(room_code);
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::GuestConnecting {
            room: room_code.clone(),
        };
        drop(state);
    }
    log::info!(
        "Attempting to join room as guest with public servers: {:?}",
        PUBLIC_SERVERS
    );
    // 计算并启动 EasyTier 实例
    let config = room_code.compute_arguments_guest(PUBLIC_SERVERS);
    // 直接创建 NetworkInstance
    let mut instance = NetworkInstance::new(config, ConfigFileControl::STATIC_CONFIG);
    instance.start().map_err(|e| {
        log::error!("Failed to start EasyTier instance: {}", e);
        ScaffoldingError::EasyTierError(e.to_string())
    })?;
    let instance = Arc::new(instance);
    
    // 等待 API 服务就绪
    std::thread::sleep(std::time::Duration::from_millis(1500));
    let mut api_service = None;
    for _ in 0..20 {
        if let Some(service) = instance.get_api_service() {
            api_service = Some(service);
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    if api_service.is_none() {
        log::error!("Failed to get API service after retries");
        if let Some(notifier) = instance.get_stop_notifier() {
            notifier.notify_one();
        }
        return Err(ScaffoldingError::EasyTierError("API service not available".to_string()));
    }
    log::info!("EasyTier instance started and API service ready");
    
    // 更新状态为 GuestStarting
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::GuestStarting {
            room: room_code.clone(),
            easytier: instance.clone(),
            difficulty: ConnectionDifficulty::Unknown,
        };
    }
    
    // 获取 NetworkInstance 引用（通过服务）
    // 发现 scaffolding 服务器
    // 获取 EasyTier 成员列表
    let members = instance.get_members().await
        .ok_or_else(|| ScaffoldingError::Other("Failed to get EasyTier members".to_string()))?;
    log::info!("EasyTier members: {:?}", members);
    
    // 查找 scaffolding 服务器主机
    let scaffolding_server = members.iter()
        .find(|member| member.hostname.starts_with("scaffolding-mc-server-"))
        .ok_or_else(|| ScaffoldingError::HostNotFound)?;
    
    // 解析端口
    let server_port = parse_hostname(&scaffolding_server.hostname)
        .ok_or_else(|| ScaffoldingError::Other("Invalid scaffolding server hostname".to_string()))?;
    
    // 获取服务器 IP 地址
    let server_ip = scaffolding_server.address
        .ok_or_else(|| ScaffoldingError::Other("Scaffolding server has no IP address".to_string()))?;
    
    log::info!("Found scaffolding server: {}:{}", server_ip, server_port);
    
    // 添加端口转发（本地端口随机选择）
    use crate::scaffolding::easytier::PortForward;
    use easytier::proto::common::SocketType;
    use std::net::{Ipv4Addr, SocketAddrV4, SocketAddr};
    let local_port = 0; // 让 EasyTier 选择本地端口
    let local_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), local_port));
    let remote_addr = SocketAddr::V4(SocketAddrV4::new(server_ip, server_port));
    let forward = PortForward {
        local: local_addr,
        remote: remote_addr,
        socket_type: SocketType::Tcp,
    };
    
    // 添加端口转发规则
    instance.add_port_forward(&[forward]).await
        .map_err(|e| ScaffoldingError::EasyTierError(e.to_string()))?;
    
    // TODO: 需要获取实际的本地端口（可能需要查询 EasyTier 配置）
    // 暂时假设端口转发成功，使用 local_port（0 表示随机）
    // 实际上应该查询 EasyTier 获取分配的端口
    // 简化处理：使用 server_port 作为本地端口（实际上不行）
    // 暂时跳过，直接使用 server_ip:server_port 连接
    
    // 创建 ProtocolClient 连接到 scaffolding 服务器
    let client = ProtocolClient::new(&format!("{}:{}", server_ip, server_port));
    
    // 测试连接
    client.ping(b"hello").await
        .map_err(|e| ScaffoldingError::ProtocolError(e))?;
    log::info!("Scaffolding server ping successful");
    
    // 协商协议
    let supported_protocols = vec!["c:ping", "c:protocols", "c:server_port", "c:player_ping", "c:player_profiles_list"];
    let server_protocols = client.negotiate_protocols(&supported_protocols).await
        .map_err(|e| ScaffoldingError::ProtocolError(e))?;
    log::info!("Server supports protocols: {:?}", server_protocols);
    
    // 获取 Minecraft 服务器端口
    let mc_port = client.get_server_port().await
        .map_err(|e| ScaffoldingError::ProtocolError(e))?;
    log::info!("Minecraft server port: {}", mc_port);
    
    // 添加 Minecraft 服务器端口转发
    let mc_local_addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), 0));
    let mc_remote_addr = SocketAddr::V4(SocketAddrV4::new(server_ip, mc_port));
    let mc_tcp_forward = PortForward {
        local: mc_local_addr,
        remote: mc_remote_addr,
        socket_type: SocketType::Tcp,
    };
    let mc_udp_forward = PortForward {
        local: mc_local_addr,
        remote: mc_remote_addr,
        socket_type: SocketType::Udp,
    };
    
    instance.add_port_forward(&[mc_tcp_forward, mc_udp_forward]).await
        .map_err(|e| ScaffoldingError::EasyTierError(e.to_string()))?;
    log::info!("Minecraft server port forwarding set up");
    
    // 更新状态为 GuestOk
    {
        let mut state = terracotta_state.lock().await;
        *state = TerracottaState::GuestOk {
            room: room_code.clone(),
            easytier: instance,
            player_profiles: Vec::new(),
        };
    }
    
    // TODO: 启动心跳任务
    log::info!("Guest joined room successfully");
    Ok(())
}

#[tauri::command]
pub async fn shutdown_room(
    terracotta_state: State<'_, Arc<tokio::sync::Mutex<TerracottaState>>>,
) -> Result<String, ScaffoldingError> {
    log::info!("Shutting down room");
    let mut state = terracotta_state.lock().await;
    let room = state.try_shutdown_current().map_err(|e| {
        log::error!("Failed to shutdown room: {}", e);
        ScaffoldingError::EasyTierError(e.to_string())
    })?;
    *state = TerracottaState::Idle;
    Ok(room.code.clone())
}
