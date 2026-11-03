use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::minecraft::instance::McInstance;
use crate::utils::platform::OS;

pub fn launcher_minecraft_directory() -> Option<PathBuf> {
    let executable = tauri::utils::platform::current_exe().ok()?;
    let appimage = std::env::var_os("APPIMAGE").map(PathBuf::from);
    let os = OS::current();
    if os == OS::macOS {
        return None;
    }
    // An AppImage's executable lives in a temporary, read-only mount.
    let executable = if os == OS::Linux {
        appimage
            .as_deref()
            .filter(|path| path.is_absolute() && path.is_file())
            .unwrap_or(&executable)
    } else {
        &executable
    };
    Some(executable.parent()?.join(".minecraft"))
}

pub fn official_minecraft_directory() -> Option<PathBuf> {
    let dirs = directories::BaseDirs::new()?;
    Some(match OS::current() {
        OS::Windows => dirs.config_dir().join(".minecraft"),
        OS::Linux => dirs.home_dir().join(".minecraft"),
        OS::macOS => dirs
            .home_dir()
            .join("Library/Application Support/minecraft"),
    })
}

pub fn create_default_directory(
    config_dir: &Path,
    portable: Option<&Path>,
) -> std::io::Result<PathBuf> {
    let prepare = |path: &Path| -> std::io::Result<PathBuf> {
        fs::create_dir_all(path.join("versions"))?;
        tempfile::tempfile_in(path)?;
        path.canonicalize()
    };
    if let Some(path) = portable {
        match prepare(path) {
            Ok(path) => return Ok(path),
            Err(error) => log::warn!(
                "Cannot use launcher-local Minecraft directory {}: {error}; using config directory",
                path.display()
            ),
        }
    }
    prepare(&config_dir.join(".minecraft"))
}

pub fn discover_repositories(
    known: &[McFolder],
    portable: Option<&Path>,
    official: Option<&Path>,
) -> Vec<McFolder> {
    let mut discovered: Vec<McFolder> = Vec::new();
    for (name, path, needs_versions) in [
        (".minecraft", portable, false),
        ("Official Launcher", official, true),
    ] {
        let Some(path) = path
            .filter(|path| path.is_dir() && (!needs_versions || path.join("versions").is_dir()))
        else {
            continue;
        };
        let Ok(path) = path.canonicalize() else {
            continue;
        };
        if known.iter().chain(&discovered).any(|repo| {
            repo.path == path
                || repo
                    .path
                    .canonicalize()
                    .is_ok_and(|existing| existing == path)
        }) {
            continue;
        }
        discovered.push(McFolder {
            name: name.into(),
            path,
            kind: McFolderType::Original,
        });
    }
    discovered
}

/// Provenance of a Minecraft folder, mirroring CE `ModFolder.McFolder.Types`.
#[derive(serde::Serialize, serde::Deserialize, Clone, Copy, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum McFolderType {
    Original,
    RenamedOriginal,
    #[default]
    Custom,
}

/// A `.minecraft` folder, not a single version folder.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct McFolder {
    pub name: String,
    pub path: PathBuf,
    #[serde(default)]
    pub kind: McFolderType,
}

impl McFolder {
    pub fn new(name: &str, path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            path,
            kind: McFolderType::Custom,
        }
    }
    /// Scan on request: installs and external launchers can change this directory.
    pub fn game_instances(&self) -> Vec<McInstance> {
        let versions_folder = self.path.join("versions");
        let entries = match fs::read_dir(&versions_folder) {
            Ok(entries) => entries,
            Err(error) => {
                log::debug!("Cannot scan {}: {error}", versions_folder.display());
                return Vec::new();
            }
        };
        let mut instances = Vec::new();
        for entry in entries {
            let entry = match entry {
                Ok(entry) => entry,
                Err(error) => {
                    log::warn!("Cannot read instance entry: {error}");
                    continue;
                }
            };
            if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                continue;
            }
            match McInstance::from_version_folder(&entry.path(), self) {
                Ok(instance) => instances.push(instance),
                Err(error) => log::debug!(
                    "Skipping incomplete or invalid instance {}: {error:?}",
                    entry.path().display()
                ),
            }
        }
        instances.sort_by(|a, b| a.name.cmp(&b.name));
        instances
    }

    pub fn game_instance_named(&self, name: &str) -> Option<McInstance> {
        self.game_instances()
            .into_iter()
            .find(|instance| instance.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_uses_directory_name_when_metadata_ids_match() {
        let root =
            std::env::temp_dir().join(format!("pcl-repo-identities-{}", uuid::Uuid::new_v4()));
        for name in ["first", "second"] {
            let dir = root.join("versions").join(name);
            fs::create_dir_all(&dir).unwrap();
            fs::write(dir.join(format!("{name}.json")), r#"{"id":"same"}"#).unwrap();
            fs::write(dir.join(format!("{name}.jar")), b"jar").unwrap();
        }
        let repo = McFolder::new("test", root.clone());
        assert_eq!(repo.game_instances().len(), 2);
        let selected = repo.game_instance_named("second").unwrap();
        assert_eq!(selected.id, "same");
        assert_eq!(selected.directory, root.join("versions/second"));
        assert!(repo.game_instance_named("same").is_none());
        fs::remove_dir_all(root).unwrap();
    }
}
