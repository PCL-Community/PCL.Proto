//! 大厅公告与特性开关（对齐 PCL CE 的 Lobby Announcement）。

use crate::link::{LAUNCHER_VERSION_CODE, LOBBY_PROTOCOL_VERSION};
use serde::{Deserialize, Serialize};
use std::time::Duration;

const FETCH_TIMEOUT: Duration = Duration::from_secs(7);
const CACHE_PATH: &str = "/api/link/v2/cache.ini";
const ANNOUNCE_PATH: &str = "/api/link/v2/announce.json";

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum LinkNoticeType {
    Notice,
    Warning,
    Important,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkNotice {
    #[serde(rename = "type")]
    pub kind: LinkNoticeType,
    pub content: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct LinkRelay {
    pub name: String,
    pub url: String,
    pub official: bool,
}

/// 服务端下发的联机大厅状态与公告。
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LinkAnnouncement {
    pub available: bool,
    pub allow_custom_name: bool,
    pub require_login: bool,
    pub require_realname: bool,
    pub protocol_version: f64,
    pub update_required: bool,
    /// 未配置公告服务器时按「本地可用」处理，保留 Terracotta 联机能力。
    pub offline: bool,
    pub notices: Vec<LinkNotice>,
    pub relays: Vec<LinkRelay>,
    pub error: Option<String>,
}

impl LinkAnnouncement {
    fn offline() -> Self {
        Self {
            available: true,
            allow_custom_name: true,
            require_login: false,
            require_realname: false,
            protocol_version: f64::from(LOBBY_PROTOCOL_VERSION),
            update_required: false,
            offline: true,
            notices: Vec::new(),
            relays: Vec::new(),
            error: None,
        }
    }

    fn failed(message: String) -> Self {
        Self {
            available: false,
            allow_custom_name: true,
            require_login: false,
            require_realname: false,
            protocol_version: f64::from(LOBBY_PROTOCOL_VERSION),
            update_required: false,
            offline: false,
            notices: Vec::new(),
            relays: Vec::new(),
            error: Some(message),
        }
    }
}

/// 一次公告拉取的结果。
pub struct FetchResult {
    pub announcement: LinkAnnouncement,
    /// 新的公告缓存（JSON 原文 + cache.ini 版本），仅在内容变化时返回。
    pub cache: Option<(String, u64)>,
    /// 所有服务器都不可用；调用方应清空旧缓存。
    pub failed: bool,
}

pub async fn fetch(
    client: &reqwest::Client,
    server_roots: &[String],
    cached: Option<(String, u64)>,
) -> FetchResult {
    if server_roots.is_empty() {
        return FetchResult {
            announcement: LinkAnnouncement::offline(),
            cache: None,
            failed: false,
        };
    }
    let mut last_error = String::new();
    for root in server_roots {
        match fetch_from(client, root, cached.as_ref()).await {
            Ok(result) => return result,
            Err(error) => last_error = error,
        }
    }
    FetchResult {
        announcement: LinkAnnouncement::failed(if last_error.is_empty() {
            "no lobby server configured".to_string()
        } else {
            last_error
        }),
        cache: None,
        failed: true,
    }
}

async fn fetch_from(
    client: &reqwest::Client,
    root: &str,
    cached: Option<&(String, u64)>,
) -> Result<FetchResult, String> {
    let version_text = client
        .get(format!("{root}{CACHE_PATH}"))
        .header(reqwest::header::USER_AGENT, user_agent())
        .timeout(FETCH_TIMEOUT)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .text()
        .await
        .map_err(|error| error.to_string())?;
    let cache_version = version_text
        .trim()
        .parse::<u64>()
        .map_err(|error| format!("invalid cache version: {error}"))?;

    if let Some((cached_json, cached_version)) = cached
        && *cached_version == cache_version
        && let Ok(announcement) = parse(cached_json)
    {
        return Ok(FetchResult {
            announcement,
            cache: None,
            failed: false,
        });
    }

    let received = client
        .get(format!("{root}{ANNOUNCE_PATH}"))
        .header(reqwest::header::USER_AGENT, user_agent())
        .timeout(FETCH_TIMEOUT)
        .send()
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?
        .text()
        .await
        .map_err(|error| error.to_string())?;
    Ok(FetchResult {
        announcement: parse(&received)?,
        cache: Some((received, cache_version)),
        failed: false,
    })
}

fn parse(json: &str) -> Result<LinkAnnouncement, String> {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Document {
        version: f64,
        available: bool,
        allow_custom_name: bool,
        require_login: bool,
        require_realname: bool,
        #[serde(default)]
        notices: Vec<serde_json::Value>,
        #[serde(default)]
        relays: Vec<serde_json::Value>,
    }
    let value: Document =
        serde_json::from_str(json).map_err(|error| format!("invalid announcement: {error}"))?;
    if !value.version.is_finite() || value.version < 0.0 {
        return Err("invalid announcement version".into());
    }
    let mut announcement = LinkAnnouncement {
        available: value.available,
        allow_custom_name: value.allow_custom_name,
        require_login: value.require_login,
        require_realname: value.require_realname,
        protocol_version: value.version,
        update_required: value.version > f64::from(LOBBY_PROTOCOL_VERSION),
        offline: false,
        notices: Vec::new(),
        relays: Vec::new(),
        error: None,
    };
    if announcement.update_required {
        announcement.available = false;
        return Ok(announcement);
    }

    for notice in &value.notices {
        let Some(content) = notice.get("content").and_then(serde_json::Value::as_str) else {
            continue;
        };
        let min_version = notice
            .get("minVer")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(0.0);
        let max_version = notice
            .get("maxVer")
            .and_then(serde_json::Value::as_f64)
            .unwrap_or(f64::MAX);
        let version_code = *LAUNCHER_VERSION_CODE as f64;
        if version_code < min_version || version_code > max_version {
            continue;
        }
        let kind = match notice
            .get("type")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default()
            .to_ascii_lowercase()
            .as_str()
        {
            "important" | "red" => LinkNoticeType::Important,
            "warning" | "yellow" => LinkNoticeType::Warning,
            _ => LinkNoticeType::Notice,
        };
        for line in content.lines() {
            let line = line.trim();
            if !line.is_empty() {
                announcement.notices.push(LinkNotice {
                    kind,
                    content: line.to_string(),
                });
            }
        }
    }

    for relay in &value.relays {
        let (Some(name), Some(url)) = (
            relay.get("name").and_then(serde_json::Value::as_str),
            relay.get("url").and_then(serde_json::Value::as_str),
        ) else {
            continue;
        };
        announcement.relays.push(LinkRelay {
            name: name.to_string(),
            url: url.to_string(),
            official: relay.get("type").and_then(serde_json::Value::as_str) == Some("official"),
        });
    }

    Ok(announcement)
}

fn user_agent() -> String {
    format!("PCL.Proto/{}", env!("CARGO_PKG_VERSION"))
}
