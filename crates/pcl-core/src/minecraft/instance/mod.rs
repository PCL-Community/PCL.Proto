pub mod files;
pub mod folder;
pub mod info;

pub use info::{McInstanceInfo, McInstanceState, instance_state};

use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{Context, ensure};
use serde_json::Value;

use crate::{
    io::net::http::api_client::game::McVersion,
    minecraft::{
        instance::folder::McFolder, java::JavaEntry, launch::launcher::GameLaunchError,
        mod_loader::ModLoaderType,
    },
};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum InstanceJava {
    Default,
    Custom(Arc<JavaEntry>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct McInstance {
    pub id: String,
    pub name: String,
    pub directory: PathBuf,
    pub jar_path: PathBuf,
    pub version: String,
    pub json_path: PathBuf,
    pub natives_path: PathBuf,
    pub game_java: InstanceJava,
    pub global_dir: PathBuf,
    pub plugin_type: ModLoaderType,
    #[serde(default, rename = "pluginsVersion")]
    pub plugins_version: BTreeMap<String, String>,
}

#[derive(Debug)]
// 三个变体都以 InvalidVersion 开头是有意的：它们都表示“这个版本装不起来”。
#[allow(clippy::enum_variant_names)]
pub enum McInstanceError {
    InvalidVersionFolder,
    InvalidVersionJson,
    InvalidVersionJar,
}

/// Rename an instance folder, its version JSON and self jar, like CE does.
pub fn rename_instance_files(
    old_directory: &Path,
    new_directory: &Path,
    old_name: &str,
    new_name: &str,
) -> Result<(), String> {
    // Only case changes need an intermediate hop on case-insensitive file systems.
    let case_only = old_name.to_lowercase() == new_name.to_lowercase();
    let temp_directory = new_directory
        .parent()
        .ok_or("Invalid instance parent")?
        .join(format!("{new_name}_pcl_rename"));
    let target = if case_only {
        &temp_directory
    } else {
        new_directory
    };
    std::fs::rename(old_directory, target).map_err(|error| error.to_string())?;
    if case_only && let Err(error) = std::fs::rename(target, new_directory) {
        let _ = std::fs::rename(target, old_directory);
        return Err(error.to_string());
    }
    let old_json = new_directory.join(format!("{old_name}.json"));
    let new_json = new_directory.join(format!("{new_name}.json"));
    if old_json.is_file() {
        let mut value: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&old_json).map_err(|error| error.to_string())?)
                .map_err(|error| error.to_string())?;
        value["id"] = new_name.into();
        std::fs::write(
            &new_json,
            serde_json::to_vec(&value).map_err(|error| error.to_string())?,
        )
        .map_err(|error| error.to_string())?;
        if !case_only {
            std::fs::remove_file(&old_json).map_err(|error| error.to_string())?;
        }
    }
    if !case_only {
        let old_jar = new_directory.join(format!("{old_name}.jar"));
        if old_jar.is_file() {
            std::fs::rename(&old_jar, new_directory.join(format!("{new_name}.jar")))
                .map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}

impl McInstance {
    pub fn shortcut_directory(&self, game_directory: &Path, kind: McInstanceFolder) -> PathBuf {
        match kind {
            McInstanceFolder::Instance => self.directory.clone(),
            McInstanceFolder::Saves => game_directory.join("saves"),
            McInstanceFolder::Mods => game_directory.join("mods"),
        }
    }
    pub fn refresh(&self) -> Result<Self, McInstanceError> {
        let repo = McFolder::new("", self.global_dir.clone());
        Self::from_version_folder(&self.directory, &repo)
    }

    pub fn from_version_folder(
        version_folder: &Path,
        repo: &McFolder,
    ) -> Result<Self, McInstanceError> {
        let folder_name = version_folder
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(McInstanceError::InvalidVersionFolder)?;
        let json_path = version_folder.join(format!("{folder_name}.json"));
        if json_path.is_file() {
            let json_reader =
                fs::File::open(&json_path).map_err(|_| McInstanceError::InvalidVersionJson)?;
            let json_content: serde_json::Value = serde_json::from_reader(json_reader)
                .map_err(|_| McInstanceError::InvalidVersionJson)?;
            let id = json_content["id"]
                .as_str()
                .ok_or(McInstanceError::InvalidVersionJson)?;
            let chain = match read_version_chain(&json_path, &repo.path) {
                Ok(chain) => chain,
                Err(error) if version_folder.join(format!("{folder_name}.jar")).is_file() => {
                    log::debug!(
                        "Cannot resolve instance inheritance {}: {error:#}",
                        json_path.display()
                    );
                    vec![(json_path.clone(), json_content.clone())]
                }
                Err(_) => return Err(McInstanceError::InvalidVersionJson),
            };
            let version = {
                let patches = json_content["patches"].as_array();
                if let Some(patches) = patches {
                    let game_patch = patches.iter().find(|patch| patch["id"] == "game");
                    if let Some(game_patch) = game_patch {
                        game_patch["version"].as_str()
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            let version = version
                .or_else(|| json_content["clientVersion"].as_str())
                .or_else(|| {
                    chain.last().and_then(|(_, value)| {
                        value["clientVersion"]
                            .as_str()
                            .or_else(|| value["id"].as_str())
                    })
                })
                .unwrap_or(id)
                .to_string();
            let jar_name = chain.iter().find_map(|(_, value)| value["jar"].as_str());
            let jar_path = if let Some(jar_name) = jar_name {
                crate::io::files::validate_file_name(jar_name)
                    .map_err(|_| McInstanceError::InvalidVersionJar)?;
                if jar_name == folder_name {
                    version_folder.join(format!("{jar_name}.jar"))
                } else {
                    repo.path
                        .join("versions")
                        .join(jar_name)
                        .join(format!("{jar_name}.jar"))
                }
            } else if uses_forge_instance_jar(&chain, &version) {
                json_path.with_extension("jar")
            } else {
                chain
                    .iter()
                    .rev()
                    .map(|(path, _)| path.with_extension("jar"))
                    .find(|path| path.is_file())
                    .ok_or(McInstanceError::InvalidVersionJar)?
            };
            if !jar_path.is_file() {
                return Err(McInstanceError::InvalidVersionJar);
            }
            let (plugin_type, plugins_version) = detect_loaders(&chain, &version);
            Ok(McInstance {
                id: id.to_string(),
                name: folder_name.to_string(),
                directory: version_folder.to_path_buf(),
                jar_path,
                version,
                json_path,
                natives_path: version_folder.join("natives"),
                game_java: InstanceJava::Default,
                global_dir: repo.path.clone(),
                plugin_type,
                plugins_version,
            })
        } else {
            log::error!("version json not found in folder: {:?}", version_folder);
            Err(McInstanceError::InvalidVersionJson)
        }
    }

    /// Loader-version view mirroring CE `McInstance.Info`.
    pub fn info(&self) -> McInstanceInfo {
        McInstanceInfo::from_versions(&self.name, &self.version, &self.plugins_version)
    }

    /// Primary instance state mirroring CE `McInstance.state`.
    pub fn state(&self) -> McInstanceState {
        let metadata = match self.resolved_metadata() {
            Ok(metadata) => metadata,
            Err(_) => return McInstanceState::Error,
        };
        let category =
            crate::minecraft::classifier::McVersionClassifier::classify_version(&metadata);
        let kind = match category {
            crate::minecraft::classifier::McVersionCategory::Release => "release",
            crate::minecraft::classifier::McVersionCategory::Snapshot => "snapshot",
            crate::minecraft::classifier::McVersionCategory::BeforeRelease => "old",
            crate::minecraft::classifier::McVersionCategory::AprilFools => "special",
        };
        instance_state(kind, metadata["releaseTime"].as_str(), &self.info())
    }

    /// CE `McInstance.Modable`.
    pub fn modable(&self) -> bool {
        self.info().modable()
    }

    pub fn read_version_json(&self) -> Result<McVersion, GameLaunchError> {
        let value = self
            .resolved_metadata()
            .map_err(|error| GameLaunchError::InvalidMetadata(format!("{error:#}")))?;
        Ok(serde_json::from_value(value)?)
    }

    pub fn resolved_metadata(&self) -> anyhow::Result<Value> {
        let chain = read_version_chain(&self.json_path, &self.global_dir)?;
        merge_metadata(chain.iter().map(|(_, value)| value))
    }
}

pub(crate) fn merge_metadata<'a>(
    chain: impl IntoIterator<Item = &'a Value>,
) -> anyhow::Result<Value> {
    fn fill_missing(child: &mut Value, parent: &Value) {
        if parent.is_null() {
            return;
        }
        if child.is_null() {
            *child = parent.clone();
        } else if let (Some(child), Some(parent)) = (child.as_object_mut(), parent.as_object()) {
            for (key, value) in parent {
                fill_missing(child.entry(key).or_insert(Value::Null), value);
            }
        }
    }
    let mut merged = Value::Object(Default::default());
    let mut libraries = Vec::new();
    let mut game = Vec::new();
    let mut jvm = Vec::new();
    let mut has_modern_game = false;
    for value in chain {
        let object = value
            .as_object()
            .context("Version metadata must be an object")?;
        for (key, field) in object {
            if !matches!(key.as_str(), "libraries" | "arguments") && !field.is_null() {
                fill_missing(
                    merged
                        .as_object_mut()
                        .unwrap()
                        .entry(key)
                        .or_insert(Value::Null),
                    field,
                );
            }
        }
        if let Some(items) = value.get("libraries").filter(|items| !items.is_null()) {
            libraries.extend(
                items
                    .as_array()
                    .context("Invalid libraries list")?
                    .iter()
                    .cloned(),
            );
        }
        if let Some(arguments) = value
            .get("arguments")
            .filter(|arguments| !arguments.is_null())
        {
            let arguments = arguments.as_object().context("Invalid arguments object")?;
            for (key, target) in [("game", &mut game), ("jvm", &mut jvm)] {
                if let Some(items) = arguments.get(key).filter(|items| !items.is_null()) {
                    if key == "game"
                        && !items
                            .as_array()
                            .context("Invalid argument list")?
                            .is_empty()
                    {
                        has_modern_game = true;
                    }
                    target.extend(
                        items
                            .as_array()
                            .context("Invalid argument list")?
                            .iter()
                            .cloned(),
                    );
                }
            }
        }
        if value
            .pointer("/arguments/game")
            .and_then(Value::as_array)
            .is_none_or(Vec::is_empty)
            && let Some(legacy) = value
                .get("minecraftArguments")
                .filter(|legacy| !legacy.is_null())
        {
            let legacy = legacy.as_str().context("Invalid legacy game arguments")?;
            game.extend(
                shlex::split(legacy)
                    .context("Invalid legacy game arguments")?
                    .into_iter()
                    .map(Value::String),
            );
        }
    }
    merged["libraries"] = Value::Array(libraries);
    if has_modern_game || !jvm.is_empty() {
        if !has_modern_game {
            game.clear();
        }
        merged["arguments"] = serde_json::json!({"game": game, "jvm": jvm});
    }
    Ok(merged)
}

#[derive(Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum McInstanceFolder {
    Instance,
    Saves,
    Mods,
}

pub fn prepare_shortcut_directory(
    instance: &McInstance,
    game_directory: &Path,
    kind: McInstanceFolder,
) -> std::io::Result<PathBuf> {
    let directory = instance.shortcut_directory(game_directory, kind);
    if !instance.directory.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Instance directory is unavailable",
        ));
    }
    if !matches!(kind, McInstanceFolder::Instance) {
        match fs::create_dir(&directory) {
            Ok(()) => (),
            Err(error)
                if error.kind() == std::io::ErrorKind::AlreadyExists && directory.is_dir() => {}
            Err(error) => return Err(error),
        }
    }
    Ok(directory)
}

fn neoforge_argument_version(metadata: &Value) -> Option<&str> {
    metadata
        .pointer("/arguments/game")?
        .as_array()?
        .windows(2)
        .find_map(|pair| {
            (pair[0].as_str()? == "--fml.neoForgeVersion")
                .then(|| pair[1].as_str())
                .flatten()
                .filter(|version| !version.is_empty() && !version.starts_with('-'))
        })
}

fn detect_loaders(
    chain: &[(PathBuf, Value)],
    minecraft: &str,
) -> (ModLoaderType, BTreeMap<String, String>) {
    let mut found = BTreeMap::new();
    let mut legacy = false;
    for (_, metadata) in chain {
        if let Some(version) = neoforge_argument_version(metadata) {
            found
                .entry("neoforge".into())
                .or_insert_with(|| version.into());
        }
        for library in metadata["libraries"].as_array().into_iter().flatten() {
            let Some(name) = library["name"].as_str() else {
                continue;
            };
            let mut parts = name.split(':');
            let (Some(group), Some(artifact), Some(version)) =
                (parts.next(), parts.next(), parts.next())
            else {
                continue;
            };
            let group = group.to_ascii_lowercase();
            let artifact = artifact.to_ascii_lowercase();
            if group == "net.legacyfabric" && artifact == "intermediary" {
                legacy = true;
            }
            let key = match (group.as_str(), artifact.as_str()) {
                ("net.fabricmc", "fabric-loader") => Some("fabric"),
                ("org.quiltmc", "quilt-loader") => Some("quilt"),
                ("net.minecraftforge", "forge" | "fmlloader" | "minecraftforge") => Some("forge"),
                ("net.neoforged", "neoforge" | "forge") => Some("neoforge"),
                ("optifine", "optifine") => Some("optifine"),
                ("com.cleanroommc", "cleanroom") => Some("cleanroom"),
                ("com.mumfrey", "liteloader") => Some("liteloader"),
                _ => None,
            };
            if let Some(key) = key {
                let version = version
                    .strip_prefix(&format!("{minecraft}-"))
                    .unwrap_or(version);
                let version = version.split_once("+build.").map_or(version, |(v, _)| v);
                if !version.is_empty() {
                    found
                        .entry(key.to_string())
                        .or_insert_with(|| version.to_string());
                }
            }
        }
        for patch in metadata["patches"].as_array().into_iter().flatten() {
            let Some(id) = patch["id"].as_str() else {
                continue;
            };
            let key = match id.to_ascii_lowercase().as_str() {
                "fabric" | "fabric-loader" => Some("fabric"),
                "legacyfabric" => Some("legacyfabric"),
                "forge" => Some("forge"),
                "neoforge" => Some("neoforge"),
                "quilt" => Some("quilt"),
                "optifine" => Some("optifine"),
                "liteloader" => Some("liteloader"),
                "cleanroom" => Some("cleanroom"),
                "labymod" => Some("laby-mod"),
                _ => None,
            };
            if let (Some(key), Some(version)) =
                (key, patch["version"].as_str().filter(|v| !v.is_empty()))
            {
                found
                    .entry(key.to_string())
                    .or_insert_with(|| version.to_string());
            }
        }
        if let Some(version) = metadata["labymod_data"]["version"]
            .as_str()
            .filter(|v| !v.is_empty())
        {
            found
                .entry("laby-mod".into())
                .or_insert_with(|| version.to_string());
        }
    }
    if (legacy || found.contains_key("legacyfabric"))
        && let Some(version) = found.remove("fabric")
    {
        found.entry("legacyfabric".into()).or_insert(version);
    }
    if found.contains_key("neoforge") {
        found.remove("forge");
    }
    let primary = if found.contains_key("legacyfabric") {
        ModLoaderType::LegacyFabric
    } else if found.contains_key("fabric") {
        ModLoaderType::Fabric
    } else if found.contains_key("quilt") {
        ModLoaderType::Quilt
    } else if found.contains_key("neoforge") {
        ModLoaderType::NeoForge
    } else if found.contains_key("forge") {
        ModLoaderType::Forge
    } else if found.contains_key("optifine") {
        ModLoaderType::Optifine
    } else {
        ModLoaderType::Vanilla
    };
    (primary, found)
}

fn uses_forge_instance_jar(chain: &[(PathBuf, Value)], version: &str) -> bool {
    let forge = chain.iter().any(|(_, value)| {
        neoforge_argument_version(value).is_some()
            || value["libraries"].as_array().is_some_and(|libraries| {
                libraries.iter().any(|lib| {
                    let mut parts = lib["name"].as_str().unwrap_or("").split(':');
                    let group = parts.next().unwrap_or("");
                    let artifact = parts.next().unwrap_or("");
                    matches!(group, "net.minecraftforge" | "net.neoforged")
                        && matches!(artifact, "forge" | "neoforge")
                })
            })
            || value["patches"].as_array().is_some_and(|patches| {
                patches
                    .iter()
                    .any(|patch| matches!(patch["id"].as_str(), Some("forge" | "neoforge")))
            })
    });
    let mut parts = version.split('.');
    let release_minor = match (parts.next(), parts.next()) {
        (Some("1"), Some(minor)) => minor
            .split(|c: char| !c.is_ascii_digit())
            .next()
            .and_then(|n| n.parse::<u32>().ok()),
        _ => None,
    };
    let modern = release_minor.map_or_else(
        || {
            chain
                .iter()
                .rev()
                .find_map(|(_, value)| value["releaseTime"].as_str())
                .is_some_and(|date| date >= "2021-05-11")
        },
        |minor| minor >= 17,
    );
    forge && modern
}

fn read_version_chain(path: &Path, repo: &Path) -> anyhow::Result<Vec<(PathBuf, Value)>> {
    let mut chain = Vec::new();
    let mut seen = HashSet::new();
    let mut path = path.to_owned();
    loop {
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .context("Invalid version path")?;
        crate::io::files::validate_file_name(name)?;
        ensure!(
            chain.len() < 32 && seen.insert(name.to_owned()),
            "Cyclic or excessively deep version inheritance"
        );
        let value: Value = serde_json::from_reader(
            fs::File::open(&path).with_context(|| format!("Cannot read {}", path.display()))?,
        )?;
        ensure!(value.is_object(), "Version metadata must be an object");
        let parent = match value.get("inheritsFrom") {
            None | Some(Value::Null) => None,
            Some(Value::String(id)) if !id.is_empty() => Some(id.clone()),
            _ => anyhow::bail!("Invalid version parent"),
        };
        chain.push((path, value));
        let Some(parent) = parent else { break };
        crate::io::files::validate_file_name(&parent)?;
        path = repo
            .join("versions")
            .join(&parent)
            .join(format!("{parent}.json"));
    }
    Ok(chain)
}

#[cfg(test)]
mod inheritance_tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn instance_shortcuts_use_game_directory_and_preserve_existing_files() {
        let root = std::env::temp_dir().join(format!("pcl-shortcuts-{}", uuid::Uuid::new_v4()));
        let version = root.join("versions/test");
        fs::create_dir_all(&version).unwrap();
        fs::write(version.join("test.json"), r#"{"id":"test"}"#).unwrap();
        fs::write(version.join("test.jar"), b"jar").unwrap();
        let repo = McFolder::new("test", root.clone());
        let instance = McInstance::from_version_folder(&version, &repo).unwrap();
        assert_eq!(
            prepare_shortcut_directory(&instance, &instance.directory, McInstanceFolder::Instance)
                .unwrap(),
            version
        );
        let saves =
            prepare_shortcut_directory(&instance, &instance.directory, McInstanceFolder::Saves)
                .unwrap();
        assert_eq!(saves, instance.directory.join("saves"));
        fs::write(saves.join("existing"), b"keep").unwrap();
        assert_eq!(
            prepare_shortcut_directory(&instance, &instance.directory, McInstanceFolder::Saves)
                .unwrap(),
            saves
        );
        assert_eq!(fs::read(saves.join("existing")).unwrap(), b"keep");
        assert_eq!(
            prepare_shortcut_directory(&instance, &root, McInstanceFolder::Mods).unwrap(),
            root.join("mods")
        );
        fs::write(version.join("mods"), b"not a directory").unwrap();
        assert!(
            prepare_shortcut_directory(&instance, &instance.directory, McInstanceFolder::Mods)
                .is_err()
        );
        assert!(serde_json::from_str::<McInstanceFolder>(r#""../outside""#).is_err());
        fs::remove_dir_all(&version).unwrap();
        assert!(
            prepare_shortcut_directory(&instance, &instance.directory, McInstanceFolder::Saves)
                .is_err()
        );
        assert!(!version.exists());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn loader_detection_handles_inheritance_and_combined_loaders() {
        let chain = vec![
            (
                PathBuf::from("child"),
                json!({"libraries": [
                    {"name":"net.minecraftforge:forge:1.20.1-47.2.0"},
                    {"name":"optifine:OptiFine:1.20.1_HD_U_I6"}
                ]}),
            ),
            (
                PathBuf::from("parent"),
                json!({"libraries": [
                    {"name":"net.minecraftforge:forge:1.20.1-46.0.0"},
                    {"name":"example:forge-helper:1.0"}
                ]}),
            ),
        ];
        let (_, loaders) = detect_loaders(&chain, "1.20.1");
        assert_eq!(loaders.get("forge").unwrap(), "47.2.0");
        assert_eq!(loaders.get("optifine").unwrap(), "1.20.1_HD_U_I6");
        assert_eq!(loaders.len(), 2);
        let legacy = vec![(
            PathBuf::new(),
            json!({"libraries":[
                {"name":"net.fabricmc:fabric-loader:0.14.0+build.1"},
                {"name":"net.legacyfabric:intermediary:1.12.2"}
            ]}),
        )];
        let (_, loaders) = detect_loaders(&legacy, "1.12.2");
        assert_eq!(loaders.get("legacyfabric").unwrap(), "0.14.0");
        assert!(!loaders.contains_key("fabric"));
        let neo = vec![(
            PathBuf::new(),
            json!({"libraries":[
            {"name":"net.minecraftforge:forge:1.20.1-47.1.0"},
            {"name":"net.neoforged:forge:1.20.1-47.1.99"}
        ], "patches":[{"id":"quilt", "version":"0.25.0"}]}),
        )];
        let (_, loaders) = detect_loaders(&neo, "1.20.1");
        assert_eq!(loaders.get("neoforge").unwrap(), "47.1.99");
        assert_eq!(loaders.get("quilt").unwrap(), "0.25.0");
        assert!(!loaders.contains_key("forge"));
        let modern_neo = vec![(
            PathBuf::new(),
            json!({"libraries":[{"name":"net.neoforged.fancymodloader:loader:2.0.1@jar"}],
                "arguments":{"game":["--fml.neoForgeVersion", "20.4.30-beta"]}}),
        )];
        let (_, loaders) = detect_loaders(&modern_neo, "1.20.4");
        assert_eq!(
            loaders.get("neoforge").map(String::as_str),
            Some("20.4.30-beta")
        );
        assert!(uses_forge_instance_jar(&modern_neo, "1.20.4"));
    }

    #[test]
    fn forge_client_jar_follows_version_and_explicit_jar() {
        let root = std::env::temp_dir().join(format!("pcl-forge-jar-{}", uuid::Uuid::new_v4()));
        let parent = root.join("versions/1.20.1");
        let child = root.join("versions/Forge");
        fs::create_dir_all(&parent).unwrap();
        fs::create_dir_all(&child).unwrap();
        fs::write(parent.join("1.20.1.jar"), b"vanilla").unwrap();
        fs::write(parent.join("1.20.1.json"), r#"{"id":"1.20.1"}"#).unwrap();
        let child_path = child.join("Forge.json");
        let mut metadata = json!({"id":"Forge","inheritsFrom":"1.20.1",
            "libraries":[{"name":"net.minecraftforge:forge:1.20.1-47.0.0"}]});
        let repo = McFolder::new("test", root.clone());
        fs::write(&child_path, serde_json::to_vec(&metadata).unwrap()).unwrap();
        assert!(matches!(
            McInstance::from_version_folder(&child, &repo),
            Err(McInstanceError::InvalidVersionJar)
        ));
        fs::write(child.join("Forge.jar"), b"patched").unwrap();
        assert_eq!(
            McInstance::from_version_folder(&child, &repo)
                .unwrap()
                .jar_path,
            child.join("Forge.jar")
        );
        metadata["jar"] = json!("1.20.1");
        fs::write(&child_path, serde_json::to_vec(&metadata).unwrap()).unwrap();
        assert_eq!(
            McInstance::from_version_folder(&child, &repo)
                .unwrap()
                .jar_path,
            parent.join("1.20.1.jar")
        );
        let mut chain = vec![(child_path, metadata)];
        assert!(!uses_forge_instance_jar(&chain, "1.16.5"));
        chain[0].1["libraries"] = json!([{"name":"net.neoforged:neoforge:21.1.0"}]);
        assert!(uses_forge_instance_jar(&chain, "1.21.1"));
        chain[0].1["releaseTime"] = json!("2021-05-12T00:00:00Z");
        assert!(uses_forge_instance_jar(&chain, "21w19a"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inherited_instance_resolves_parent_jar_metadata_and_rejects_unsafe_chains() {
        let root = std::env::temp_dir().join(format!("pcl-inheritance-{}", uuid::Uuid::new_v4()));
        let parent = root.join("versions/1.20.1");
        let child = root.join("versions/Fabric");
        fs::create_dir_all(&parent).unwrap();
        fs::create_dir_all(&child).unwrap();
        fs::write(parent.join("1.20.1.jar"), b"jar").unwrap();
        let artifact = json!({"sha1":"a".repeat(40),"size":1,"url":"https://example.org/client"});
        fs::write(parent.join("1.20.1.json"), serde_json::to_vec(&json!({
            "id":"1.20.1","type":"release","mainClass":"Vanilla", "assets":"legacy",
            "assetIndex":{"id":"legacy","sha1":"a".repeat(40),"size":1,"url":"https://example.org/index"},
            "downloads":{"client":artifact},"javaVersion":{"majorVersion":17},
            "libraries":[{"name":"example:base:1"}],
            "minecraftArguments":"--username ${auth_player_name} --title 'Legacy Title'"
        })).unwrap()).unwrap();
        let child_path = child.join("Fabric.json");
        let child_metadata = json!({"id":"Fabric","inheritsFrom":"1.20.1","mainClass":"FabricMain",
            "libraries":[{"name":"example:loader:2"}],"arguments":{"game":["--loader","fabric"],"jvm":["-cp","${classpath}"]}});
        fs::write(&child_path, serde_json::to_vec(&child_metadata).unwrap()).unwrap();
        let repo = McFolder::new("test", root.clone());
        let instance = McInstance::from_version_folder(&child, &repo).unwrap();
        let mut old = serde_json::to_value(&instance).unwrap();
        old.as_object_mut().unwrap().remove("pluginsVersion");
        assert!(
            serde_json::from_value::<McInstance>(old)
                .unwrap()
                .plugins_version
                .is_empty()
        );
        assert_eq!(instance.jar_path, parent.join("1.20.1.jar"));
        assert_eq!(instance.version, "1.20.1");
        let resolved = instance.resolved_metadata().unwrap();
        assert_eq!(resolved["mainClass"], "FabricMain");
        assert_eq!(resolved["javaVersion"]["majorVersion"], 17);
        assert_eq!(resolved["libraries"][0]["name"], "example:loader:2");
        assert_eq!(resolved["libraries"][1]["name"], "example:base:1");
        assert_eq!(
            resolved["arguments"]["game"],
            json!([
                "--loader",
                "fabric",
                "--username",
                "${auth_player_name}",
                "--title",
                "Legacy Title"
            ])
        );
        assert_eq!(
            instance.read_version_json().unwrap().main_class,
            "FabricMain"
        );
        fs::write(child.join("Fabric.jar"), b"local jar").unwrap();
        assert_eq!(
            McInstance::from_version_folder(&child, &repo)
                .unwrap()
                .jar_path,
            parent.join("1.20.1.jar")
        );
        for parent_id in ["../outside", "Fabric"] {
            let mut bad = child_metadata.clone();
            bad["inheritsFrom"] = parent_id.into();
            fs::write(&child_path, serde_json::to_vec(&bad).unwrap()).unwrap();
            assert!(instance.resolved_metadata().is_err());
        }
        let incomplete = McInstance::from_version_folder(&child, &repo).unwrap();
        assert!(incomplete.resolved_metadata().is_err());
        fs::remove_dir_all(root).unwrap();
    }
}

#[cfg(test)]
mod rename_tests {
    use super::*;

    #[test]
    fn renames_folder_version_json_and_self_jar() {
        let root = std::env::temp_dir().join(format!("pcl-rename-{}", uuid::Uuid::new_v4()));
        let versions = root.join("versions");
        let old = versions.join("Old Name");
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(
            old.join("Old Name.json"),
            br#"{"id":"Old Name","type":"release"}"#,
        )
        .unwrap();
        std::fs::write(old.join("Old Name.jar"), b"client").unwrap();
        let new = versions.join("New Name");
        rename_instance_files(&old, &new, "Old Name", "New Name").unwrap();
        assert!(!old.exists());
        assert!(!new.join("Old Name.json").exists());
        assert!(!new.join("Old Name.jar").exists());
        assert!(new.join("New Name.jar").is_file());
        let json: serde_json::Value =
            serde_json::from_slice(&std::fs::read(new.join("New Name.json")).unwrap()).unwrap();
        assert_eq!(json["id"], "New Name");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn rejects_missing_source_without_leaving_files_behind() {
        let root = std::env::temp_dir().join(format!("pcl-rename-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).unwrap();
        assert!(
            rename_instance_files(
                &root.join("versions/Missing"),
                &root.join("versions/New"),
                "Missing",
                "New"
            )
            .is_err()
        );
        assert!(!root.join("versions/New").exists());
        std::fs::remove_dir_all(root).unwrap();
    }
}
