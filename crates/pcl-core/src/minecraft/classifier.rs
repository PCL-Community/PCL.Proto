//! Minecraft version classification, mirroring
//! `PCL.Core.Minecraft.McVersionClassifier`.
//!
//! Proto keeps the classifier pure and language-neutral. Display names and
//! April-Fools lore text belong to the frontend localization layer.

use chrono::{DateTime, Utc};
use serde_json::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McVersionCategory {
    Release,
    Snapshot,
    BeforeRelease,
    AprilFools,
}

pub struct McVersionClassifier;

#[allow(dead_code)]
impl McVersionClassifier {
    pub fn classify_version(version: &Value) -> McVersionCategory {
        Self::classify_parts(
            version["id"].as_str().unwrap_or_default(),
            version["type"].as_str().unwrap_or_default(),
        )
    }

    /// Classify from already-parsed fields, avoiding a temporary JSON value.
    pub fn classify_parts(id: &str, kind: &str) -> McVersionCategory {
        let id = id.to_ascii_lowercase();
        match kind {
            "release" => McVersionCategory::Release,
            "special" => McVersionCategory::AprilFools,
            "snapshot" | "pending" => Self::classify_snapshot_or_pending(&id),
            _ => McVersionCategory::BeforeRelease,
        }
    }

    pub fn release_time(version: &Value) -> DateTime<Utc> {
        version["releaseTime"]
            .as_str()
            .and_then(|value| DateTime::parse_from_rfc3339(value).ok())
            .map(|value| value.with_timezone(&Utc))
            .unwrap_or(DateTime::<Utc>::MIN_UTC)
    }

    fn classify_snapshot_or_pending(id: &str) -> McVersionCategory {
        if id.starts_with("1.")
            && !id.contains("combat")
            && !id.contains("rc")
            && !id.contains("experimental")
            && id != "1.2"
            && !id.contains("pre")
        {
            McVersionCategory::Release
        } else if is_april_fools(id) {
            McVersionCategory::AprilFools
        } else {
            McVersionCategory::Snapshot
        }
    }
}

/// Known April-Fools identifiers, matching CE's classifier.
pub fn is_april_fools(id: &str) -> bool {
    let id = id.to_ascii_lowercase();
    matches!(
        id.as_str(),
        "2point0_blue"
            | "2point0_red"
            | "2point0_purple"
            | "2.0_blue"
            | "2.0_red"
            | "2.0_purple"
            | "2.0"
            | "20w14infinite"
            | "20w14∞"
            | "3d shareware v1.34"
            | "1.rv-pre1"
            | "15w14a"
            | "22w13oneblockatatime"
            | "23w13a_or_b"
            | "24w14potato"
            | "25w14craftmine"
            | "26w14a"
    ) || id.starts_with("2.0")
        || id.starts_with("2point0")
        || id.starts_with("20w14inf")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn classifies_releases_snapshots_old_and_fool_versions() {
        assert_eq!(
            McVersionClassifier::classify_version(&json!({"id":"1.20.1","type":"release"})),
            McVersionCategory::Release
        );
        assert_eq!(
            McVersionClassifier::classify_version(&json!({"id":"1.21","type":"snapshot"})),
            McVersionCategory::Release
        );
        assert_eq!(
            McVersionClassifier::classify_version(&json!({"id":"23w13a","type":"snapshot"})),
            McVersionCategory::Snapshot
        );
        assert_eq!(
            McVersionClassifier::classify_version(&json!({"id":"a1.2.6","type":"old_alpha"})),
            McVersionCategory::BeforeRelease
        );
        assert_eq!(
            McVersionClassifier::classify_version(&json!({"id":"2.0_red","type":"special"})),
            McVersionCategory::AprilFools
        );
        assert!(is_april_fools("20w14infinite"));
        assert!(is_april_fools("24w14potato"));
        assert!(!is_april_fools("1.20.1"));
    }
}
