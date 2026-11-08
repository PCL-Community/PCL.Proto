//! Discover Minecraft LAN worlds.
//!
//! Mirrors PCL CE's `BroadcastListener` plus per-port status ping: listen on the
//! vanilla multicast channel for a short window, keep advertisements from local
//! interfaces only, then query every advertised game port.

use crate::link::mcping;
use futures_util::{StreamExt, stream};
use serde::Serialize;
use socket2::{Domain, Socket, Type};
use std::collections::BTreeSet;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddrV4, SocketAddrV6, UdpSocket};
use std::time::{Duration, Instant};

const DISCOVERY_PORT: u16 = 4445;
const MULTICAST_V4: Ipv4Addr = Ipv4Addr::new(224, 0, 2, 60);
const MULTICAST_V6: Ipv6Addr = Ipv6Addr::new(0xff75, 0x230, 0, 0, 0, 0, 0, 0x60);
const PING_TIMEOUT: Duration = Duration::from_secs(3);

/// One Minecraft world currently shared to the local network.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredWorld {
    pub port: u16,
    /// MOTD as plain text (the structured chat component is flattened).
    pub description: String,
    /// Version name reported by the server, e.g. `1.21.4`.
    pub version: String,
    pub latency_ms: u64,
}

/// Listen for LAN advertisements for `window` and probe every advertised port.
pub async fn discover_worlds(window: Duration) -> Result<Vec<DiscoveredWorld>, String> {
    let ports = tokio::task::spawn_blocking(move || listen_ports(window))
        .await
        .map_err(|error| error.to_string())??;
    let mut worlds: Vec<_> = stream::iter(ports)
        .map(query_world)
        .buffer_unordered(16)
        .filter_map(|world| async { world.ok() })
        .collect()
        .await;
    worlds.sort_by_key(|world| world.port);
    Ok(worlds)
}

/// Probe the same local MC endpoint used by discovery and manual port input.
pub async fn query_world(port: u16) -> Result<DiscoveredWorld, String> {
    let (result, latency) =
        mcping::server_query_with_timeout(&format!("127.0.0.1:{port}"), PING_TIMEOUT).await?;
    if result.version.protocol == 0 {
        return Err("not a Minecraft server".into());
    }
    Ok(DiscoveredWorld {
        port,
        description: chat_text(&result.description),
        version: result.version.name,
        latency_ms: latency as u64,
    })
}

fn listen_ports(window: Duration) -> Result<Vec<u16>, String> {
    let local = local_addresses();
    let sockets: Vec<UdpSocket> = [build_v4(&local), build_v6()]
        .into_iter()
        .flatten()
        .collect();
    if sockets.is_empty() {
        return Err("no LAN discovery socket available".into());
    }
    let deadline = Instant::now() + window;
    let mut ports = BTreeSet::new();
    let mut buffer = [0u8; 8192];
    while Instant::now() < deadline {
        for socket in &sockets {
            let Ok((length, sender)) = socket.recv_from(&mut buffer) else {
                continue;
            };
            if !is_local(sender.ip(), &local) {
                continue;
            }
            let Ok(text) = std::str::from_utf8(&buffer[..length]) else {
                continue;
            };
            if let Some(port) = advertised_port(text) {
                ports.insert(port);
            }
        }
    }
    Ok(ports.into_iter().collect())
}

fn build_v4(local: &[IpAddr]) -> Option<UdpSocket> {
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None).ok()?;
    socket.set_reuse_address(true).ok()?;
    socket
        .bind(&SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, DISCOVERY_PORT).into())
        .ok()?;
    // 组播加入失败时仍保留套接字：部分平台只允许指定接口加入组播，
    // 而单播的局域网公告依然会到达。
    for address in local {
        if let IpAddr::V4(address) = address {
            let _ = socket.join_multicast_v4(&MULTICAST_V4, address);
        }
    }
    let _ = socket.join_multicast_v4(&MULTICAST_V4, &Ipv4Addr::UNSPECIFIED);
    socket
        .set_read_timeout(Some(Duration::from_millis(150)))
        .ok()?;
    Some(socket.into())
}

fn build_v6() -> Option<UdpSocket> {
    let socket = Socket::new(Domain::IPV6, Type::DGRAM, None).ok()?;
    socket.set_only_v6(true).ok()?;
    socket.set_reuse_address(true).ok()?;
    socket
        .bind(&SocketAddrV6::new(Ipv6Addr::UNSPECIFIED, DISCOVERY_PORT, 0, 0).into())
        .ok()?;
    let _ = socket.join_multicast_v6(&MULTICAST_V6, 0);
    socket
        .set_read_timeout(Some(Duration::from_millis(150)))
        .ok()?;
    Some(socket.into())
}

fn local_addresses() -> Vec<IpAddr> {
    local_ip_address::list_afinet_netifas()
        .map(|interfaces| interfaces.into_iter().map(|(_, address)| address).collect())
        .unwrap_or_default()
}

fn is_local(address: IpAddr, local: &[IpAddr]) -> bool {
    address.is_loopback() || local.contains(&address)
}

fn advertised_port(text: &str) -> Option<u16> {
    let (_, motd) = text.split_once("[MOTD]")?;
    let (motd, _) = motd.split_once("[/MOTD]")?;
    if terracotta::is_lobby_motd(motd) {
        return None;
    }
    let (_, rest) = text.split_once("[AD]")?;
    let (port, _) = rest.split_once("[/AD]")?;
    port.trim().parse::<u16>().ok().filter(|port| *port >= 1024)
}

/// Flatten a Minecraft chat component (or plain string) to its text.
pub fn chat_text(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Array(items) => items.iter().map(chat_text).collect(),
        serde_json::Value::Object(object) => {
            let mut text = object
                .get("text")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_default()
                .to_owned();
            if let Some(extra) = object.get("extra") {
                text.push_str(&chat_text(extra));
            }
            text
        }
        _ => String::new(),
    }
}
