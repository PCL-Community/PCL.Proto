use crate::scaffolding;
use easytier::launcher::NetworkInstance;
use easytier::proto::api::config::{
    ConfigPatchAction, InstanceConfigPatch, PatchConfigRequest, PortForwardPatch,
};
use easytier::proto::api::instance::ShowNodeInfoRequest;
use easytier::proto::common::{PortForwardConfigPb, SocketType};
use easytier::proto::{
    api::instance::ListRouteRequest, common::NatType, rpc_types::controller::BaseController,
};
use std::iter::once;
use std::net::{Ipv4Addr, SocketAddr};
use std::str::FromStr;
use std::time::Duration;

/// 连接难度，根据EasyTier确定的网络结构转换而来
pub enum ConnectionDifficulty {
    Unknown,
    Easiest,
    Simple,
    Medium,
    Tough,
}

impl From<(&NatType, &NatType)> for ConnectionDifficulty {
    fn from(value: (&NatType, &NatType)) -> Self {
        let is = |types: &[NatType]| -> bool { types.contains(value.0) || types.contains(value.1) };
        if is(&[NatType::Unknown]) {
            ConnectionDifficulty::Unknown
        } else if is(&[NatType::OpenInternet]) {
            ConnectionDifficulty::Easiest
        } else if is(&[NatType::NoPat, NatType::FullCone]) {
            ConnectionDifficulty::Simple
        } else if is(&[NatType::Restricted, NatType::PortRestricted]) {
            ConnectionDifficulty::Medium
        } else {
            ConnectionDifficulty::Tough
        }
    }
}

/// EasyTier 成员 不知道和PlayerProfile有什么区别
pub struct EasyTierMember {
    pub hostname: String,
    pub address: Option<Ipv4Addr>,
    pub is_local: bool,
    pub nat: NatType,
}

pub struct PortForward{
    local: SocketAddr,
    remote: SocketAddr,
    socket_type: SocketType,
}

pub trait EasyTierControl {
    /// 获取 EasyTier 实例的成员列表
    async fn get_members(&self) -> Option<Vec<EasyTierMember>>;

    /// 把 EasyTier 协议中的 IPv4 地址转换为 Rust 标准库中的 `Ipv4Addr` 类型
    fn parse_address(address: Option<easytier::proto::common::Ipv4Inet>) -> Option<Ipv4Addr>;

    /// 启动后台监控线程
    fn launch_monitor_thread(&self, room_code: &str, port: u16);

    /// 添加端口转发
    async fn add_port_forward(&self, forwards: &[PortForward]) -> anyhow::Result<()>;
}

impl EasyTierControl for std::sync::Arc<NetworkInstance> {
    async fn get_members(&self) -> Option<Vec<EasyTierMember>> {
        let api_service = self.get_api_service()?;
        let service = api_service.get_peer_manage_service();
        let (neighbours, this) = (
            service
                .list_route(BaseController::default(), ListRouteRequest::default())
                .await
                .ok()
                .map(|response| response.routes)?,
            service
                .show_node_info(BaseController::default(), ShowNodeInfoRequest::default())
                .await
                .ok()
                .map(|response| response.node_info)??,
        );
        Some(
            neighbours
                .into_iter()
                .map(|route| EasyTierMember {
                    hostname: route.hostname,
                    address: Self::parse_address(route.ipv4_addr),
                    nat: route
                        .stun_info
                        .map_or(NatType::Unknown, |info| info.udp_nat_type()),
                    is_local: false,
                })
                .chain(once(EasyTierMember {
                    hostname: this.hostname,
                    address: Ipv4Addr::from_str(&this.ipv4_addr).ok(),
                    is_local: true,
                    nat: this
                        .stun_info
                        .map_or(NatType::Unknown, |info| info.udp_nat_type()),
                }))
                .collect::<Vec<_>>(),
        )
    }

    fn parse_address(address: Option<easytier::proto::common::Ipv4Inet>) -> Option<Ipv4Addr> {
        address
            .and_then(|address| address.address)
            .map(|address| Ipv4Addr::from_octets(address.addr.to_be_bytes()))
    }

    fn launch_monitor_thread(&self, room_code: &str, port: u16) {
        let instance_arc = self.clone();
        let room_code = room_code.to_string();
        std::thread::spawn(move || {
            let mut counter = 0;
            let mut easytier_retry_counter = 0;
            const MAX_EASYTIER_RETRIES: u8 = 3;

            log::info!("Starting monitoring thread for room: {}", room_code);

            loop {
                std::thread::sleep(Duration::from_secs(5));

                // 检查 Minecraft 服务器连接
                if scaffolding::mc::check_mc_connection(port) {
                    counter = 0;
                    log::debug!(
                        "Minecraft server connection check passed for room: {}",
                        room_code
                    );
                } else {
                    counter += 1;
                    log::warn!(
                        "Minecraft server connection check failed (attempt {}/3) for room: {}",
                        counter,
                        room_code
                    );
                    if counter >= 3 {
                        // 连接失败，处理错误
                        log::error!(
                            "Minecraft server connection failed after 3 attempts for room: {}",
                            room_code
                        );
                        break;
                    }
                }

                // 检查 EasyTier 实例状态
                if !instance_arc.is_easytier_running() {
                    easytier_retry_counter += 1;
                    log::error!(
                        "EasyTier instance is not running (retry {}/{}) for room: {}",
                        easytier_retry_counter,
                        MAX_EASYTIER_RETRIES,
                        room_code
                    );

                    if easytier_retry_counter >= MAX_EASYTIER_RETRIES {
                        log::error!(
                            "EasyTier instance failed after {} retries for room: {}",
                            MAX_EASYTIER_RETRIES,
                            room_code
                        );
                        break;
                    }

                    // 尝试重新启动 EasyTier
                    log::info!("Attempting to restart EasyTier for room: {}", room_code);
                    // 这里可以添加重新启动逻辑
                } else {
                    easytier_retry_counter = 0;
                    log::debug!(
                        "EasyTier instance is running normally for room: {}",
                        room_code
                    );
                }

                // [TODO] 添加更多监控逻辑，比如清理超时的客户端
            }

            log::error!("Monitoring thread stopped for room: {}", room_code);
        });
    }

    async fn add_port_forward(&self, forwards: &[PortForward]) -> anyhow::Result<()> {
        let service = self
            .get_api_service()
            .ok_or_else(|| anyhow::anyhow!("Failed to get API service"))?;
        let task = service.get_config_service().patch_config(
            BaseController::default(),
            PatchConfigRequest {
                patch: Some(InstanceConfigPatch {
                    port_forwards: forwards
                        .iter()
                        .map(|forward| PortForwardPatch {
                            action: ConfigPatchAction::Add as i32,
                            cfg: Some(PortForwardConfigPb {
                                bind_addr: Some(forward.local.into()),
                                dst_addr: Some(forward.remote.into()),
                                socket_type: forward.socket_type.into(),
                            }),
                        })
                        .collect::<Vec<PortForwardPatch>>(),
                    ..Default::default()
                }),
                ..Default::default()
            },
        );
        task.await?;
        Ok(())
    }
}
