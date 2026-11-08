//! 联机大厅服务：公告、账户、设置、世界发现与 Terracotta 状态聚合。

use crate::config::link::LinkSettings;
use crate::config::manager;
use crate::link::LinkError;
use crate::link::announcement::{self, LinkAnnouncement};
use crate::link::naid::{self, LinkAccount};
use pcl_core::link::{discovery, scaffolding};
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, Mutex};
use std::time::Duration;
use tokio::sync::Mutex as AsyncMutex;

const DISCOVERY_WINDOW: Duration = Duration::from_secs(3);
/// 串行化启动、退出和配置变更，避免已撤销的操作重新启动大厅。
pub static CONTROL: AsyncMutex<()> = AsyncMutex::const_new(());

static ANNOUNCEMENT: LazyLock<Mutex<Option<LinkAnnouncement>>> = LazyLock::new(|| Mutex::new(None));

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkInit {
    pub settings: LinkSettings,
    pub eula: bool,
    pub announcement: LinkAnnouncement,
    pub account: LinkAccount,
    pub meta: serde_json::Value,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkPlayer {
    pub name: String,
    #[serde(alias = "machine_id")]
    pub machine_id: String,
    pub vendor: String,
    pub kind: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkLobbyState {
    pub state: String,
    pub index: u64,
    pub room: Option<String>,
    pub url: Option<String>,
    #[serde(alias = "latency")]
    pub latency_ms: Option<u64>,
    #[serde(alias = "connection_type")]
    pub connection_type: Option<scaffolding::ConnectionType>,
    #[serde(default, alias = "profiles")]
    pub players: Vec<LinkPlayer>,
    #[serde(alias = "type")]
    pub exception: Option<u8>,
}

/// 发现/校验到的本地 MC 世界，直接使用 pcl-core 的形态。
pub use pcl_core::link::discovery::DiscoveredWorld as LinkWorld;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkNatStatus {
    pub udp: String,
    pub tcp: String,
    pub ipv6: bool,
}

fn apply_vendor() {
    scaffolding::set_vendor(format!("PCL.Proto {}", env!("CARGO_PKG_VERSION")));
}

fn apply_settings(settings: &LinkSettings) {
    scaffolding::set_options(settings.options);
}

fn apply_relays(announcement: &LinkAnnouncement) {
    scaffolding::set_public_nodes(
        announcement
            .relays
            .iter()
            .map(|relay| relay.url.clone())
            .collect(),
    );
}

fn store_announcement(announcement: &LinkAnnouncement) {
    *ANNOUNCEMENT.lock().unwrap() = Some(announcement.clone());
}

fn cached_announcement() -> Option<LinkAnnouncement> {
    ANNOUNCEMENT.lock().unwrap().clone()
}

async fn fetch_announcement(force: bool) -> Result<LinkAnnouncement, LinkError> {
    let state = manager::get_link_state().await;
    let cached = if force {
        None
    } else {
        state
            .announce_cache
            .clone()
            .map(|json| (json, state.announce_cache_version))
    };
    let client = manager::ConfigManager::instance().api_client.http_client();
    let result = announcement::fetch(&client, &crate::secrets::link_server_roots(), cached).await;
    if result.failed {
        let _ = manager::update_link_state(|state| {
            state.clear_announcement_cache();
            Ok(())
        })
        .await;
    } else if let Some((json, version)) = result.cache {
        let _ = manager::update_link_state(move |state| {
            state.announce_cache = Some(json.clone());
            state.announce_cache_version = version;
            Ok(())
        })
        .await;
    }
    apply_relays(&result.announcement);
    store_announcement(&result.announcement);
    Ok(result.announcement)
}

pub async fn refresh_announcement() -> Result<LinkAnnouncement, LinkError> {
    let _control = CONTROL.lock().await;
    fetch_announcement(true).await
}

pub async fn initialize() -> Result<LinkInit, LinkError> {
    let _control = CONTROL.lock().await;
    apply_vendor();
    let settings = manager::get_link_settings().await;
    apply_settings(&settings);
    let announcement = fetch_announcement(false).await?;
    let account = naid::initialize().await;
    let state = manager::get_link_state().await;
    Ok(LinkInit {
        settings,
        eula: state.eula_accepted,
        announcement,
        account,
        meta: scaffolding::get_terracotta_meta(),
    })
}

pub async fn settings() -> LinkSettings {
    manager::get_link_settings().await
}

pub async fn update_settings(settings: LinkSettings) -> Result<LinkSettings, LinkError> {
    let _control = CONTROL.lock().await;
    let saved = manager::set_link_settings(settings)
        .await
        .map_err(LinkError::from)?;
    apply_settings(&saved);
    Ok(saved)
}

pub async fn reset_settings() -> Result<LinkSettings, LinkError> {
    let _control = CONTROL.lock().await;
    naid::reset_runtime().await;
    let settings = manager::reset_link_settings()
        .await
        .map_err(LinkError::from)?;
    apply_settings(&settings);
    *ANNOUNCEMENT.lock().unwrap() = None;
    scaffolding::set_public_nodes(Vec::new());
    stop_inner().await?;
    Ok(settings)
}

pub async fn set_eula(accepted: bool) -> Result<bool, LinkError> {
    let _control = CONTROL.lock().await;
    if !accepted {
        naid::reset_runtime().await;
    }
    manager::update_link_state(move |state| {
        state.eula_accepted = accepted;
        if !accepted {
            state.clear_credentials();
        }
        Ok(())
    })
    .await
    .map_err(LinkError::from)?;
    if !accepted {
        *ANNOUNCEMENT.lock().unwrap() = None;
        scaffolding::set_public_nodes(Vec::new());
        stop_inner().await?;
    }
    Ok(accepted)
}

fn read_lobby_state() -> Result<LinkLobbyState, LinkError> {
    serde_json::from_value(scaffolding::get_terracotta_state()).map_err(|error| {
        log::error!("Invalid Terracotta lobby state: {error}");
        LinkError::new("lobby_failed")
    })
}

pub async fn lobby_state() -> Result<LinkLobbyState, LinkError> {
    tokio::task::spawn_blocking(read_lobby_state)
        .await
        .map_err(|_| LinkError::new("lobby_failed"))?
}

pub async fn nat_status() -> Result<LinkNatStatus, LinkError> {
    let raw = tokio::task::spawn_blocking(scaffolding::test_nat)
        .await
        .map_err(|_| LinkError::new("nat_failed"))?
        .ok_or_else(|| LinkError::new("nat_failed"))?;
    Ok(LinkNatStatus {
        udp: nat_name(raw.udp_nat_type),
        tcp: nat_name(raw.tcp_nat_type),
        ipv6: raw.ipv6,
    })
}

fn nat_name(code: i32) -> String {
    match code {
        0 => "open",
        1 => "no_pat",
        2 => "full_cone",
        3 => "restricted",
        4 => "port_restricted",
        5 => "symmetric_easy",
        6 => "symmetric",
        7 => "symmetric_firewall",
        8 => "udp_blocked",
        _ => "unknown",
    }
    .to_string()
}

pub async fn discover_worlds() -> Result<Vec<LinkWorld>, LinkError> {
    discovery::discover_worlds(DISCOVERY_WINDOW)
        .await
        .map_err(|error| {
            log::warn!("World discovery failed: {error}");
            LinkError::new("discover_failed")
        })
}

pub async fn verify_port(port: i64) -> Result<LinkWorld, LinkError> {
    let port = u16::try_from(port)
        .ok()
        .filter(|port| *port >= 1024)
        .ok_or_else(|| LinkError::new("invalid_port"))?;
    discovery::query_world(port)
        .await
        .map_err(|_| LinkError::new("not_mc_port"))
}

fn ensure_runtime_ready() -> Result<(), LinkError> {
    use crate::commands::link_runtime::{LinkRuntimeState, get_link_runtime_status};
    if get_link_runtime_status().state == LinkRuntimeState::Ready {
        Ok(())
    } else {
        Err(LinkError::new("easy_tier_unavailable"))
    }
}

fn precheck(announcement: &LinkAnnouncement, account: &LinkAccount) -> Result<(), LinkError> {
    if announcement.update_required {
        return Err(LinkError::new("update_required"));
    }
    if !announcement.available {
        return Err(LinkError::new("lobby_unavailable"));
    }
    if announcement.require_login {
        if !account.logged_in {
            return Err(LinkError::new("login_required"));
        }
        if account.expired
            || account.error.is_some()
            || account.pending
            || account.username.is_none()
            || account.status.is_none()
        {
            return Err(LinkError::new("relogin_required"));
        }
        if announcement.require_realname && !account.realname {
            return Err(LinkError::new("realname_required"));
        }
        if account.status.is_some_and(|status| status != 0) {
            return Err(LinkError::new("account_banned"));
        }
    }
    ensure_runtime_ready()
}

fn resolved_username(
    settings: &LinkSettings,
    account: &LinkAccount,
    allow_custom_name: bool,
) -> Result<String, LinkError> {
    let custom = settings.username.trim();
    let name = if allow_custom_name && !custom.is_empty() {
        custom.to_string()
    } else {
        account.username.clone().unwrap_or_default()
    };
    if name.trim().is_empty() {
        return Err(LinkError::new("username_required"));
    }
    if name.contains('|') || name.chars().any(char::is_control) || name.chars().count() > 64 {
        return Err(LinkError::new("invalid_username"));
    }
    Ok(name)
}

async fn prepare() -> Result<(LinkSettings, LinkAnnouncement, LinkAccount), LinkError> {
    if !manager::get_link_state().await.eula_accepted {
        return Err(LinkError::new("eula_required"));
    }
    let settings = manager::get_link_settings().await;
    settings.validate().map_err(LinkError::from)?;
    apply_vendor();
    apply_settings(&settings);
    let announcement = match cached_announcement() {
        Some(announcement) => announcement,
        None => fetch_announcement(false).await?,
    };
    let current_account = naid::account().await;
    let account = if announcement.require_login && current_account.logged_in {
        naid::refresh()
            .await
            .map_err(|_| LinkError::new("relogin_required"))?
    } else {
        current_account
    };
    Ok((settings, announcement, account))
}

pub async fn create(port: i64) -> Result<LinkLobbyState, LinkError> {
    let _control = CONTROL.lock().await;
    let (settings, announcement, account) = prepare().await?;
    precheck(&announcement, &account)?;
    let world = verify_port(port).await?;
    let username = resolved_username(&settings, &account, announcement.allow_custom_name)?;
    tokio::task::spawn_blocking(move || {
        scaffolding::set_host_starting(world.port, username)
            .map_err(|_| LinkError::new("create_failed"))?;
        read_lobby_state()
    })
    .await
    .map_err(|_| LinkError::new("create_failed"))?
}

pub async fn join(code: String) -> Result<LinkLobbyState, LinkError> {
    let _control = CONTROL.lock().await;
    let code = code.trim().to_uppercase();
    if code.is_empty() {
        return Err(LinkError::new("invalid_code"));
    }
    let (settings, announcement, account) = prepare().await?;
    precheck(&announcement, &account)?;
    let username = resolved_username(&settings, &account, announcement.allow_custom_name)?;
    tokio::task::spawn_blocking(move || {
        scaffolding::set_guesting(code, username).map_err(LinkError::from)?;
        read_lobby_state()
    })
    .await
    .map_err(|_| LinkError::new("join_failed"))?
}

async fn stop_inner() -> Result<LinkLobbyState, LinkError> {
    tokio::task::spawn_blocking(|| {
        scaffolding::set_waiting();
        read_lobby_state()
    })
    .await
    .map_err(|_| LinkError::new("lobby_failed"))?
}

pub async fn stop() -> Result<LinkLobbyState, LinkError> {
    let _control = CONTROL.lock().await;
    stop_inner().await
}
