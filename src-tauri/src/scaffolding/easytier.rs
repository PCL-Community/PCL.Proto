use easytier::proto::common::NatType;
use std::net::Ipv4Addr;

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
        if is(&[NatType::OpenInternet]) {
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
