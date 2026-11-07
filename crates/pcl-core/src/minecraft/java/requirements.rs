//! Version constraints ported from CE ModLaunch.McLaunchJava, using normalized versions.
use super::JavaVersion;
use crate::minecraft::instance::McInstance;
use anyhow::ensure;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, io::Read};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JavaRequirements {
    pub min: JavaVersion,
    pub max: JavaVersion,
    pub compatible_majors: BTreeSet<u32>,
    pub component: Option<String>,
}
impl JavaRequirements {
    pub fn accepts(&self, version: JavaVersion) -> bool {
        version >= self.min
            && version <= self.max
            && (self.compatible_majors.is_empty() || self.compatible_majors.contains(&version.0))
    }
    pub fn description(&self) -> String {
        if !self.compatible_majors.is_empty() {
            format!(
                "Java {} [{} – {}]",
                self.compatible_majors
                    .iter()
                    .map(u32::to_string)
                    .collect::<Vec<_>>()
                    .join(", "),
                self.min,
                self.max
            )
        } else {
            format!("Java {} – {}", self.min, self.max)
        }
    }
    pub fn download_major(&self) -> Option<u32> {
        let major = if self.compatible_majors.is_empty() {
            self.min.0.max(8)
        } else {
            *self.compatible_majors.iter().find(|m| {
                JavaVersion::upper(**m) >= self.min && JavaVersion::major(**m) <= self.max
            })?
        };
        // CE asks for manual installation for Java 7 and restricted Java 8 update ranges.
        if major < 8
            || (major == 8
                && (self.min > JavaVersion::major(8) || self.max < JavaVersion::upper(8)))
        {
            None
        } else {
            Some(major)
        }
    }
    pub fn for_instance(instance: &McInstance) -> anyhow::Result<Self> {
        let mut metadata = instance.resolved_metadata()?;
        // Vanilla client JAR version.json also carries java_version/java_component in recent releases.
        if let Ok(file) = std::fs::File::open(&instance.jar_path)
            && let Ok(mut jar) = zip::ZipArchive::new(file)
            && let Ok(mut entry) = jar.by_name("version.json")
        {
            ensure!(
                entry.size() <= 1024 * 1024,
                "Oversized version.json in client JAR"
            );
            let mut content = String::new();
            entry.read_to_string(&mut content)?;
            let version: Value = serde_json::from_str(&content)?;
            for field in ["java_version", "java_component"] {
                if metadata[field].is_null() {
                    metadata[field] = version[field].clone();
                }
            }
        }
        Ok(Self::from_metadata(&metadata, &instance.version))
    }
    pub fn from_metadata(meta: &Value, fallback_version: &str) -> Self {
        let version = meta["patches"]
            .as_array()
            .and_then(|p| p.iter().find(|p| p["id"] == "game"))
            .and_then(|p| p["version"].as_str())
            .or_else(|| meta["clientVersion"].as_str())
            .unwrap_or(fallback_version);
        let parts: Vec<u32> = version
            .split(['.', '-', ' '])
            .take(3)
            .map_while(|p| p.parse().ok())
            .collect();
        let vanilla = (parts.first() == Some(&1) && parts.len() >= 2)
            .then(|| (parts[1], parts.get(2).copied().unwrap_or(0)));
        let release = meta["releaseTime"].as_str().unwrap_or("");
        let mut min = JavaVersion::major(0);
        let mut max = JavaVersion::upper(999);
        if vanilla.is_some_and(|v| v >= (20, 5)) || (vanilla.is_none() && release >= "2024-04-02") {
            min = JavaVersion::major(21);
        } else if vanilla.is_some_and(|v| v.0 >= 18)
            || (vanilla.is_none() && release >= "2021-11-16")
        {
            min = JavaVersion::major(17);
        } else if vanilla.is_some_and(|v| v.0 >= 17)
            || (vanilla.is_none() && release >= "2021-05-11")
        {
            min = JavaVersion::major(16);
        } else if release >= "2017" || vanilla.is_some_and(|v| v.0 >= 12) {
            min = JavaVersion::major(8);
        } else if ("2001"..="2013-05-01").contains(&release) || vanilla.is_some_and(|v| v <= (5, 2))
        {
            max = JavaVersion::upper(8);
        }
        let libraries = meta["libraries"]
            .as_array()
            .map(Vec::as_slice)
            .unwrap_or(&[]);
        let patches = meta["patches"].as_array().map(Vec::as_slice).unwrap_or(&[]);
        let loader = |names: &[&str]| -> Option<String> {
            libraries
                .iter()
                .find_map(|lib| {
                    let name = lib["name"].as_str()?;
                    let mut parts = name.split(':');
                    let group = parts.next()?;
                    let artifact = parts.next()?;
                    let version = parts.next()?;
                    names
                        .iter()
                        .any(|n| group.eq_ignore_ascii_case(n) || artifact.eq_ignore_ascii_case(n))
                        .then(|| version.to_owned())
                })
                .or_else(|| {
                    patches.iter().find_map(|p| {
                        let id = p["id"].as_str()?;
                        names
                            .iter()
                            .any(|n| id.eq_ignore_ascii_case(n))
                            .then(|| p["version"].as_str().unwrap_or("0").to_owned())
                    })
                })
        };
        let optifine = loader(&["optifine"]).is_some();
        if optifine && let Some((minor, _)) = vanilla {
            if minor < 7 || minor == 12 {
                max = JavaVersion::upper(8);
            } else if (8..12).contains(&minor) {
                min = JavaVersion::major(8);
                max = JavaVersion::upper(8);
            }
        }
        if let Some(forge) = loader(&["net.minecraftforge", "forge"]) {
            match vanilla {
                Some(v) if ((6, 1)..=(7, 2)).contains(&v) => {
                    min = min.max(JavaVersion::major(7));
                    max = max.min(JavaVersion::upper(7));
                }
                None | Some((0..=12, _)) => max = JavaVersion::upper(8),
                Some((13..=14, _)) => {
                    min = min.max(JavaVersion::major(8));
                    max = max.min(JavaVersion::upper(10));
                }
                Some((15, _)) => {
                    min = min.max(JavaVersion::major(8));
                    max = max.min(JavaVersion::upper(15));
                }
                _ => {
                    let forge = forge.split_once('-').map_or(forge.as_str(), |(_, f)| f);
                    if let Some(v) = JavaVersion::parse(forge)
                        && v >= JavaVersion::major(34)
                        && v <= JavaVersion(36, 2, 25, 0)
                    {
                        max = max.min(JavaVersion(8, 0, 321, 0));
                    } else if optifine && vanilla.is_some_and(|v| v.0 == 18) {
                        max = max.min(JavaVersion::upper(18));
                    }
                }
            }
        }
        if let Some(cleanroom) = loader(&["cleanroom", "com.cleanroommc"]) {
            min = min.max(JavaVersion::major(
                if JavaVersion::parse(&cleanroom).is_some_and(|v| v < JavaVersion(0, 5, 0, 0)) {
                    21
                } else {
                    25
                },
            ));
        }
        if loader(&["fabric-loader", "net.fabricmc"]).is_some()
            && let Some((minor, _)) = vanilla
        {
            if (15..=16).contains(&minor) {
                min = min.max(JavaVersion::major(8));
            } else if minor >= 18 {
                min = min.max(JavaVersion::major(17));
            }
        }
        if loader(&["liteloader", "com.mumfrey"]).is_some() && vanilla.is_some() {
            max = max.min(JavaVersion::upper(8));
        }
        if loader(&["labymod", "net.labymod"]).is_some() {
            min = min.max(JavaVersion::major(21));
            max = JavaVersion::upper(999);
        }
        let recommended = meta["javaVersion"]["majorVersion"]
            .as_u64()
            .or_else(|| meta["java_version"].as_u64())
            .unwrap_or(0) as u32;
        if recommended > 0 {
            min = min.max(JavaVersion::major(recommended));
            if max < min {
                max = JavaVersion::upper(999);
            }
        }
        let compatible_majors: BTreeSet<u32> = meta["compatibleJavaMajors"]
            .as_array()
            .into_iter()
            .flatten()
            .filter_map(|m| m.as_u64())
            .filter(|m| *m > 0 && *m <= 999)
            .map(|m| m as u32)
            .collect();
        if let (Some(low), Some(high)) = (compatible_majors.first(), compatible_majors.last()) {
            let low = JavaVersion::major(*low);
            let high = JavaVersion::upper(*high);
            let intersection = (min.max(low), max.min(high));
            (min, max) = if intersection.0 <= intersection.1 {
                intersection
            } else {
                (low, high)
            };
        }
        Self {
            min,
            max,
            compatible_majors,
            component: meta["javaVersion"]["component"]
                .as_str()
                .or_else(|| meta["java_component"].as_str())
                .filter(|c| !c.is_empty())
                .map(str::to_owned),
        }
    }
}
