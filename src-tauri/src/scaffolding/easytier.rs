use easytier::launcher::NetworkInstance;
use easytier::proto::api::instance::ShowNodeInfoRequest;
use easytier::proto::{
    api::instance::ListRouteRequest, common::NatType, rpc_types::controller::BaseController,
};
use std::iter::once;
use std::net::Ipv4Addr;
use std::str::FromStr;

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

trait EasyTierControl {
    async fn get_members(&self) -> Option<Vec<EasyTierMember>>;

    fn parse_address(address: Option<easytier::proto::common::Ipv4Inet>) -> Option<Ipv4Addr> {
        address
            .and_then(|address| address.address)
            .map(|address| Ipv4Addr::from_octets(address.addr.to_be_bytes()))
    }
}

impl EasyTierControl for NetworkInstance {
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
}
