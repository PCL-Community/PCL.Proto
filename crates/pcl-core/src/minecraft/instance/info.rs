//! Instance metadata derived from loader versions, mirroring
//! `Plain Craft Launcher 2/Modules/Minecraft/McInstanceInfo.cs`.

use std::collections::BTreeMap;

/// Primary instance state shown in the instance list.
///
/// Mirrors CE `McInstanceState`, with Proto's `Vanilla` naming replaced by
/// `Original` to match CE's `Original` value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum McInstanceState {
    #[default]
    #[serde(rename = "error")]
    Error,
    #[serde(rename = "original")]
    Original,
    #[serde(rename = "snapshot")]
    Snapshot,
    #[serde(rename = "fool")]
    Fool,
    #[serde(rename = "optifine")]
    OptiFine,
    #[serde(rename = "old")]
    Old,
    #[serde(rename = "forge")]
    Forge,
    #[serde(rename = "neoforge")]
    NeoForge,
    #[serde(rename = "liteloader")]
    LiteLoader,
    #[serde(rename = "fabric")]
    Fabric,
    #[serde(rename = "legacyfabric")]
    LegacyFabric,
    #[serde(rename = "quilt")]
    Quilt,
    #[serde(rename = "cleanroom")]
    Cleanroom,
    #[serde(rename = "labymod")]
    LabyMod,
}

use serde::{Deserialize, Serialize};

/// Loader versions and derived capability flags for one instance.
///
/// The `plugins_version` map keeps keys such as `fabric`, `forge` and
/// `neoforge`; the `has_*` fields mirror CE's public `McInstanceInfo` shape.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct McInstanceInfo {
    pub vanilla_name: String,
    pub vanilla_version: String,
    pub fabric: String,
    pub legacy_fabric: String,
    pub quilt: String,
    pub forge: String,
    pub neo_forge: String,
    pub optifine: String,
    pub cleanroom: String,
    pub liteloader: String,
    pub labymod: String,
    pub has_fabric: bool,
    pub has_legacy_fabric: bool,
    pub has_quilt: bool,
    pub has_forge: bool,
    pub has_neoforge: bool,
    pub has_optifine: bool,
    pub has_cleanroom: bool,
    pub has_liteloader: bool,
    pub has_labymod: bool,
}

impl McInstanceInfo {
    pub fn from_versions(
        vanilla_name: impl Into<String>,
        vanilla_version: impl Into<String>,
        versions: &BTreeMap<String, String>,
    ) -> Self {
        let value = |key: &str| versions.get(key).cloned().unwrap_or_default();
        let mut info = Self {
            vanilla_name: vanilla_name.into(),
            vanilla_version: vanilla_version.into(),
            fabric: value("fabric"),
            legacy_fabric: value("legacyfabric"),
            quilt: value("quilt"),
            forge: value("forge"),
            neo_forge: value("neoforge"),
            optifine: value("optifine"),
            cleanroom: value("cleanroom"),
            liteloader: value("liteloader"),
            labymod: value("laby-mod"),
            ..Self::default()
        };
        info.has_fabric = !info.fabric.is_empty();
        info.has_legacy_fabric = !info.legacy_fabric.is_empty();
        info.has_quilt = !info.quilt.is_empty();
        info.has_forge = !info.forge.is_empty();
        info.has_neoforge = !info.neo_forge.is_empty();
        info.has_optifine = !info.optifine.is_empty();
        info.has_cleanroom = !info.cleanroom.is_empty();
        info.has_liteloader = !info.liteloader.is_empty();
        info.has_labymod = !info.labymod.is_empty();
        info
    }

    /// CE `HasForgelike`.
    pub fn has_forge_like(&self) -> bool {
        self.has_forge || self.has_neoforge || self.has_cleanroom
    }

    /// CE `HasFabriclike`.
    pub fn has_fabric_like(&self) -> bool {
        self.has_fabric || self.has_quilt || self.has_legacy_fabric
    }

    /// Whether the instance can install mods, matching CE `McInstance.Modable`.
    pub fn modable(&self) -> bool {
        self.has_forge_like() || self.has_fabric_like() || self.has_liteloader || self.has_labymod
    }
}

/// Derive the primary state from the Vanilla type and loader flags.
pub fn instance_state(
    version_type: &str,
    release_time: Option<&str>,
    info: &McInstanceInfo,
) -> McInstanceState {
    let kind = version_type.to_ascii_lowercase();
    if matches!(kind.as_str(), "fool" | "special") {
        return McInstanceState::Fool;
    }
    if matches!(kind.as_str(), "old_alpha" | "old_beta" | "old")
        || release_time
            .and_then(|time| time.get(..4))
            .and_then(|year| year.parse::<u32>().ok())
            .is_some_and(|year| (2001..2013).contains(&year))
    {
        return McInstanceState::Old;
    }
    if info.has_labymod {
        return McInstanceState::LabyMod;
    }
    if info.has_cleanroom {
        return McInstanceState::Cleanroom;
    }
    if info.has_neoforge {
        return McInstanceState::NeoForge;
    }
    if info.has_forge {
        return McInstanceState::Forge;
    }
    if info.has_legacy_fabric {
        return McInstanceState::LegacyFabric;
    }
    if info.has_fabric {
        return McInstanceState::Fabric;
    }
    if info.has_quilt {
        return McInstanceState::Quilt;
    }
    if info.has_liteloader {
        return McInstanceState::LiteLoader;
    }
    if info.has_optifine {
        return McInstanceState::OptiFine;
    }
    if matches!(kind.as_str(), "snapshot" | "pending") {
        return McInstanceState::Snapshot;
    }
    McInstanceState::Original
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_loader_flags_and_primary_state() {
        let mut versions = BTreeMap::new();
        versions.insert("fabric".into(), "0.16".into());
        versions.insert("forge".into(), "47.2".into());
        let info = McInstanceInfo::from_versions("1.20.1", "1.20.1", &versions);
        assert!(info.has_fabric);
        assert!(info.has_forge);
        assert!(info.modable());
        assert_eq!(
            instance_state("release", Some("2023-06-12T00:00:00Z"), &info),
            McInstanceState::Forge
        );
        assert_eq!(
            instance_state("snapshot", None, &McInstanceInfo::default()),
            McInstanceState::Snapshot
        );
        assert_eq!(
            instance_state("release", Some("2012-04-04T00:00:00Z"), &info),
            McInstanceState::Old
        );
    }
}
