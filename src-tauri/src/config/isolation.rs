use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context;

use pcl_core::minecraft::instance::McInstance;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IsolationDefault {
    Off,
    Modded,
    NonRelease,
    ModdedOrNonRelease,
    #[default]
    All,
}

fn contains_entries(path: &Path, files: bool) -> anyhow::Result<bool> {
    let entries = match fs::read_dir(path) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => {
            return Err(error).with_context(|| format!("Cannot inspect {}", path.display()));
        }
    };
    for entry in entries {
        let kind = entry?.file_type()?;
        if (files && kind.is_file()) || (!files && kind.is_dir()) {
            return Ok(true);
        }
    }
    Ok(false)
}

pub fn initial(instance: &McInstance, default: IsolationDefault) -> anyhow::Result<bool> {
    if contains_entries(&instance.directory.join("mods"), true)?
        || contains_entries(&instance.directory.join("saves"), false)?
    {
        return Ok(true);
    }
    if default == IsolationDefault::All {
        return Ok(true);
    }
    if default == IsolationDefault::Off {
        return Ok(false);
    }
    let modded = [
        "forge",
        "neoforge",
        "fabric",
        "legacyfabric",
        "quilt",
        "liteloader",
        "cleanroom",
        "laby-mod",
    ]
    .iter()
    .any(|name| instance.plugins_version.contains_key(*name));
    if default == IsolationDefault::Modded {
        return Ok(modded);
    }
    let metadata = instance.resolved_metadata()?;
    let kind = metadata["type"]
        .as_str()
        .context("Missing Minecraft version type")?;
    let version = instance.version.to_ascii_lowercase();
    let snapshot = kind == "snapshot"
        || kind == "pending"
        || instance.name.to_ascii_lowercase().contains("combat")
        || ["w", "snapshot", "rc", "pre", "experimental", "-"]
            .iter()
            .any(|part| version.contains(part));
    let old = matches!(kind, "old_alpha" | "old_beta" | "old")
        || metadata["releaseTime"]
            .as_str()
            .and_then(|time| time.get(..4))
            .and_then(|year| year.parse::<u32>().ok())
            .is_some_and(|year| (2001..2013).contains(&year));
    let fool = matches!(kind, "fool" | "special")
        || [
            "20w14∞",
            "3d shareware v1.34",
            "1.rv-pre1",
            "15w14a",
            "22w13oneblockatatime",
            "23w13a_or_b",
            "24w14potato",
            "25w14craftmine",
            "26w14a",
        ]
        .contains(&version.as_str())
        || version.starts_with("2.0")
        || version.starts_with("2point0")
        || version.starts_with("20w14inf");
    let loader = modded || instance.plugins_version.contains_key("optifine");
    let non_release = old || (!loader && (snapshot || fool));
    Ok(non_release || (default == IsolationDefault::ModdedOrNonRelease && modded))
}

/// 文件缺少隔离键时才初始化；显式 false 必须保留。
pub fn directory(instance: &McInstance) -> Result<PathBuf, String> {
    let config = super::instance::InstanceConfig::load(&instance.directory)?;
    if config.get("VersionArgumentIndieV2", true)? {
        Ok(instance.directory.clone())
    } else {
        Ok(instance.global_dir.clone())
    }
}

use crate::config::states::AppState;
pub fn ensure(state: &AppState, path: &Path) -> Result<bool, String> {
    let instance = super::java::active_instance(state, path)?;
    ensure_for(instance, state.isolation_default)
}

pub fn ensure_for(instance: &McInstance, default: IsolationDefault) -> Result<bool, String> {
    super::instance::InstanceConfig::update(&instance.directory, |config| {
        if config.contains("VersionArgumentIndieV2") {
            return config.get("VersionArgumentIndieV2", false);
        }
        let refreshed = instance
            .refresh()
            .map_err(|error| format!("Cannot refresh Minecraft instance: {error:?}"))?;
        let decision = initial(&refreshed, default).map_err(|error| format!("{error:#}"))?;
        config.set("VersionArgumentIndieV2", decision)?;
        Ok(decision)
    })
}

pub fn set_default(state: &mut AppState, value: IsolationDefault) -> Result<(), String> {
    if state.isolation_default == value {
        return Ok(());
    }
    for repo in &state.repositories {
        if !repo.path.is_dir() {
            return Err(format!(
                "Minecraft folder is unavailable: {}",
                repo.path.display()
            ));
        }
        let versions = repo.path.join("versions");
        let entries = match std::fs::read_dir(&versions) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
            Err(error) => return Err(format!("Cannot inspect {}: {error}", versions.display())),
        };
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            if !entry
                .file_type()
                .map_err(|error| error.to_string())?
                .is_dir()
            {
                continue;
            }
            let path = entry.path();
            if let Ok(instance) = McInstance::from_version_folder(&path, repo) {
                ensure_for(&instance, state.isolation_default)?;
            }
        }
    }
    state.isolation_default = value;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pcl_core::minecraft::instance::folder::McFolder;

    #[test]
    fn defaults_and_existing_content_determine_isolation_once_per_directory() {
        let root = std::env::temp_dir().join(format!("pcl-isolation-{}", uuid::Uuid::new_v4()));
        let path = root.join("versions/first");
        fs::create_dir_all(&path).unwrap();
        let metadata = path.join("first.json");
        fs::write(
            &metadata,
            r#"{"id":"1.20.1","type":"release","releaseTime":"2023-06-12T00:00:00Z"}"#,
        )
        .unwrap();
        fs::write(path.join("first.jar"), b"jar").unwrap();
        let repo = McFolder::new("test", root.clone());
        let mut instance = McInstance::from_version_folder(&path, &repo).unwrap();
        for (mode, expected) in [
            (IsolationDefault::Off, false),
            (IsolationDefault::Modded, false),
            (IsolationDefault::NonRelease, false),
            (IsolationDefault::ModdedOrNonRelease, false),
            (IsolationDefault::All, true),
        ] {
            assert_eq!(initial(&instance, mode).unwrap(), expected);
        }
        instance.plugins_version.insert("fabric".into(), "1".into());
        assert!(initial(&instance, IsolationDefault::Modded).unwrap());
        assert!(!initial(&instance, IsolationDefault::NonRelease).unwrap());
        assert!(initial(&instance, IsolationDefault::ModdedOrNonRelease).unwrap());
        instance.plugins_version.clear();
        fs::write(
            &metadata,
            r#"{"id":"1.2.5","type":"release","releaseTime":"2012-04-04T00:00:00Z"}"#,
        )
        .unwrap();
        assert!(initial(&instance, IsolationDefault::NonRelease).unwrap());
        fs::write(&metadata, r#"{"id":"20w14infinite","type":"snapshot"}"#).unwrap();
        instance.version = "20w14infinite".into();
        assert!(initial(&instance, IsolationDefault::NonRelease).unwrap());
        fs::create_dir(path.join("mods")).unwrap();
        fs::write(path.join("mods/mod.jar"), b"mod").unwrap();
        assert!(initial(&instance, IsolationDefault::Off).unwrap());
        fs::remove_dir_all(path.join("mods")).unwrap();
        fs::create_dir_all(path.join("saves/world")).unwrap();
        assert!(initial(&instance, IsolationDefault::Off).unwrap());
        fs::remove_dir_all(path.join("saves")).unwrap();
        assert_eq!(directory(&instance).unwrap(), path);
        super::super::instance::InstanceConfig::update(&path, |config| {
            config.set("VersionArgumentIndieV2", false)
        })
        .unwrap();
        assert_eq!(directory(&instance).unwrap(), root);
        fs::remove_dir_all(root).unwrap();
    }
}
