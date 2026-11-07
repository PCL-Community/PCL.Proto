//! Server-list ping results, mirroring `PCL.Core.Link.McPing.Model`.

use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McPingVersionResult {
    pub name: String,
    #[serde(default)]
    pub protocol: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McPingPlayerSampleResult {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub id: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McPingPlayerResult {
    #[serde(default)]
    pub max: i32,
    #[serde(default)]
    pub online: i32,
    #[serde(default, rename = "sample")]
    pub samples: Vec<McPingPlayerSampleResult>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McPingModInfoModResult {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct McPingModInfoResult {
    #[serde(default, rename = "type")]
    pub kind: String,
    #[serde(default, rename = "modList")]
    pub mod_list: Vec<McPingModInfoModResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct McPingResult {
    pub version: McPingVersionResult,
    #[serde(default)]
    pub players: McPingPlayerResult,
    /// Chat component: servers may return a string or a structured object.
    #[serde(default)]
    pub description: Value,
    #[serde(default)]
    pub favicon: Option<String>,
    #[serde(default)]
    pub latency: u64,
    #[serde(default, rename = "modinfo")]
    pub mod_info: Option<McPingModInfoResult>,
    #[serde(default)]
    pub prevents_chat_reports: Option<bool>,
}
