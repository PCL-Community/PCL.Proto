use easytier::common::config::{ConfigFileControl, TomlConfigLoader};
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

pub struct EasyTier {
    instance: Option<NetworkInstance>,
}

impl EasyTier {
    /// 启动 EasyTier 实例
    pub fn launch(&mut self, config: TomlConfigLoader) -> anyhow::Result<()> {
        let mut instance = NetworkInstance::new(config, ConfigFileControl::STATIC_CONFIG);
        instance.start().unwrap();
        self.instance = Some(instance);
        Ok(())
    }

    /// 终止 EasyTier 实例
    pub fn terminate(&mut self) -> anyhow::Result<()> {
        if let Some(instance) = self.instance.as_ref() {
            if let Some(msg) = instance.get_latest_error_msg() {
                return Err(anyhow::anyhow!(
                    "EasyTier has encountered an fatal error: {}",
                    msg
                ));
            }
            let Some(stop_notifier) = instance.get_stop_notifier() else {
                return Err(anyhow::anyhow!("Stop notifier not found"));
            };
            stop_notifier.notify_one();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Instance not found"))
        }
    }
}

impl EasyTierControl for EasyTier {
    async fn peer_list(&self) -> Option<Vec<Peer>> {
        self.instance.as_ref()?.peer_list().await
    }

    async fn add_port_forward(&self, forwards: &[PortForward]) -> anyhow::Result<()> {
        let instance = self
            .instance
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("EasyTier instance not launched"))?;
        instance.add_port_forward(forwards).await?;
        Ok(())
    }
}

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

#[derive(Debug)]
pub struct Peer {
    pub hostname: String,
    pub address: Option<Ipv4Addr>,
    pub is_local: bool,
    pub nat: NatType,
}

pub struct PortForward {
    /// 本地绑定地址
    pub local: SocketAddr,
    /// 目标地址
    pub remote: SocketAddr,
    /// 使用的协议类型 tcp/udp
    pub socket_type: SocketType,
}

pub trait EasyTierControl {
    /// 获取 EasyTier 实例的成员列表
    async fn peer_list(&self) -> Option<Vec<Peer>>;

    /// 添加端口转发
    async fn add_port_forward(&self, forwards: &[PortForward]) -> anyhow::Result<()>;
}

/// 把 EasyTier 协议中的 IPv4 地址转换为 Rust 标准库中的 `Ipv4Addr` 类型
fn parse_address(address: Option<easytier::proto::common::Ipv4Inet>) -> Option<Ipv4Addr> {
    address
        .and_then(|address| address.address)
        .map(|address| Ipv4Addr::from_octets(address.addr.to_be_bytes()))
}

impl EasyTierControl for NetworkInstance {
    async fn peer_list(&self) -> Option<Vec<Peer>> {
        let api_service = self.get_api_service()?;
        let neighbours = api_service
            .get_peer_manage_service()
            .list_route(BaseController::default(), ListRouteRequest::default())
            .await
            .ok()
            .map(|response| response.routes)?;
        let this = api_service
            .get_peer_manage_service()
            .show_node_info(BaseController::default(), ShowNodeInfoRequest::default())
            .await
            .ok()
            .map(|response| response.node_info)??;
        Some(
            neighbours
                .into_iter()
                .map(|route| Peer {
                    hostname: route.hostname,
                    address: parse_address(route.ipv4_addr),
                    nat: route
                        .stun_info
                        .map_or(NatType::Unknown, |info| info.udp_nat_type()),
                    is_local: false,
                })
                .chain(once(Peer {
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
