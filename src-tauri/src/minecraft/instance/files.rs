use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use serde::Serialize;

/// Resource folders of one instance that the instance pages can list and modify.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Mod,
    ResourcePack,
    Shader,
    Schematic,
    Screenshot,
    Save,
    Datapack,
}

impl Kind {
    pub fn parse(value: &str) -> Result<Self, String> {
        Ok(
            match value.to_ascii_lowercase().replace(['_', '-'], "").as_str() {
                "mod" => Self::Mod,
                "resourcepack" => Self::ResourcePack,
                "shader" | "shaderpack" => Self::Shader,
                "schematic" => Self::Schematic,
                "screenshot" => Self::Screenshot,
                "save" => Self::Save,
                "datapack" => Self::Datapack,
                other => return Err(format!("Unknown resource kind: {other}")),
            },
        )
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    pub name: String,
    pub path: PathBuf,
    pub size: u64,
    /// Seconds since the Unix epoch, 0 when unavailable.
    pub modified: u64,
    pub enabled: bool,
    pub icon: Option<PathBuf>,
    /// Filled by the command layer so the webview can load the picture via a scoped protocol.
    pub token: Option<String>,
}

pub fn directory(kind: Kind, game_directory: &Path, save: Option<&str>) -> Result<PathBuf, String> {
    Ok(match kind {
        Kind::Mod => game_directory.join("mods"),
        Kind::ResourcePack => game_directory.join("resourcepacks"),
        Kind::Shader => game_directory.join("shaderpacks"),
        Kind::Schematic => game_directory.join("schematics"),
        Kind::Screenshot => game_directory.join("screenshots"),
        Kind::Save => game_directory.join("saves"),
        Kind::Datapack => {
            let save = save.ok_or("A save name is required for datapacks")?;
            if save.is_empty()
                || [".", ".."].contains(&save)
                || save.contains(['/', '\\'])
                || save.chars().any(char::is_control)
            {
                return Err("Invalid save name".into());
            }
            game_directory.join("saves").join(save).join("datapacks")
        }
    })
}

pub fn list(kind: Kind, directory: &Path) -> Result<Vec<Entry>, String> {
    let reader = match fs::read_dir(directory) {
        Ok(reader) => reader,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.to_string()),
    };
    let mut entries = Vec::new();
    for item in reader {
        let path = item.map_err(|error| error.to_string())?.path();
        let metadata = match fs::metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => continue,
        };
        let Some(raw_name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        let stripped = strip_suffix(raw_name, ".disabled")
            .or_else(|| strip_suffix(raw_name, ".old"))
            .unwrap_or(raw_name);
        let disabled = stripped.len() != raw_name.len();
        let base = stripped.to_ascii_lowercase();
        let matched = match kind {
            Kind::Mod => {
                base.ends_with(".jar") || base.ends_with(".zip") || base.ends_with(".litemod")
            }
            Kind::ResourcePack | Kind::Shader => {
                metadata.is_dir() || (metadata.is_file() && base.ends_with(".zip"))
            }
            Kind::Schematic => {
                metadata.is_file()
                    && [".schematic", ".schem", ".litematic", ".nbt"]
                        .iter()
                        .any(|extension| base.ends_with(extension))
            }
            Kind::Screenshot => metadata.is_file() && base.ends_with(".png"),
            Kind::Save => metadata.is_dir() && path.join("level.dat").is_file(),
            Kind::Datapack => metadata.is_dir() || (metadata.is_file() && base.ends_with(".zip")),
        };
        if !matched {
            continue;
        }
        let name = match kind {
            Kind::Screenshot => path
                .file_stem()
                .and_then(|stem| stem.to_str())
                .unwrap_or(raw_name)
                .to_owned(),
            _ => stripped.to_owned(),
        };
        let icon = match kind {
            Kind::Save if path.join("icon.png").is_file() => Some(path.join("icon.png")),
            Kind::Screenshot => Some(path.clone()),
            _ => None,
        };
        entries.push(Entry {
            name,
            path,
            size: if metadata.is_file() {
                metadata.len()
            } else {
                0
            },
            modified: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0),
            enabled: !disabled,
            icon,
            token: None,
        });
    }
    entries.sort_by_key(|entry| entry.name.to_lowercase());
    Ok(entries)
}

fn strip_suffix<'a>(name: &'a str, suffix: &str) -> Option<&'a str> {
    if name.len() > suffix.len() && name[name.len() - suffix.len()..].eq_ignore_ascii_case(suffix) {
        Some(&name[..name.len() - suffix.len()])
    } else {
        None
    }
}

/// Enable or disable files by toggling the CE-compatible `.disabled` suffix.
pub fn set_enabled(paths: &[PathBuf], enabled: bool) -> Result<(), String> {
    for path in paths {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return Err("Invalid resource path".into());
        };
        let lower = name.to_ascii_lowercase();
        let disabled = lower.ends_with(".disabled");
        if enabled == !disabled {
            continue;
        }
        let target = if enabled {
            path.with_file_name(strip_suffix(name, ".disabled").ok_or("Resource is not disabled")?)
        } else {
            path.with_file_name(format!("{name}.disabled"))
        };
        if target.exists() {
            return Err(format!(
                "A file named {} already exists; resolve the conflict first",
                target.display()
            ));
        }
        fs::rename(path, &target).map_err(|error| error.to_string())?;
    }
    Ok(())
}

pub fn delete(paths: &[PathBuf]) -> Result<(), String> {
    for path in paths {
        if !path.exists() {
            continue;
        }
        trash::delete(path).map_err(|error| error.to_string())?;
    }
    Ok(())
}

/// Copy picked files (or unpack a save archive) into the resource folder.
pub fn import(sources: &[PathBuf], kind: Kind, directory: &Path) -> Result<(), String> {
    fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    for source in sources {
        let name = source
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or("Invalid source file name")?;
        let is_archive = kind == Kind::Save
            && source
                .extension()
                .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"));
        if is_archive {
            extract_zip(source, directory)?;
            continue;
        }
        let target = unique_path(directory.join(name));
        copy(source, &target)?;
    }
    Ok(())
}

fn copy(source: &Path, target: &Path) -> Result<(), String> {
    if source.is_dir() {
        fs::create_dir_all(target).map_err(|error| error.to_string())?;
        for item in fs::read_dir(source).map_err(|error| error.to_string())? {
            let item = item.map_err(|error| error.to_string())?;
            copy(&item.path(), &target.join(item.file_name()))?;
        }
    } else {
        fs::copy(source, target).map_err(|error| error.to_string())?;
    }
    Ok(())
}

fn extract_zip(source: &Path, directory: &Path) -> Result<(), String> {
    let mut archive = zip::ZipArchive::new(fs::File::open(source).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?;
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).map_err(|e| e.to_string())?;
        let Some(relative) = file.enclosed_name() else {
            continue;
        };
        let target = directory.join(relative);
        if file.is_dir() {
            fs::create_dir_all(&target).map_err(|e| e.to_string())?;
            continue;
        }
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(|e| e.to_string())?;
        }
        let mut output = fs::File::create(&target).map_err(|e| e.to_string())?;
        std::io::copy(&mut file, &mut output).map_err(|e| e.to_string())?;
        output.flush().map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Same-name copies get a ` (n)` suffix instead of overwriting existing files.
fn unique_path(path: PathBuf) -> PathBuf {
    if !path.exists() {
        return path;
    }
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default()
        .to_owned();
    let extension = path.extension().and_then(|value| value.to_str());
    for index in 2..1000 {
        let name = match extension {
            Some(extension) => format!("{stem} ({index}).{extension}"),
            None => format!("{stem} ({index})"),
        };
        let candidate = path.with_file_name(name);
        if !candidate.exists() {
            return candidate;
        }
    }
    path
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> PathBuf {
        std::env::temp_dir().join(format!("pcl-resources-{}", uuid::Uuid::new_v4()))
    }

    #[test]
    fn lists_mods_and_toggles_the_disabled_suffix() {
        let root = root();
        fs::create_dir_all(root.join("mods")).unwrap();
        fs::write(root.join("mods/a.jar"), b"a").unwrap();
        fs::write(root.join("mods/b.jar.disabled"), b"b").unwrap();
        fs::write(root.join("mods/notes.txt"), b"ignore").unwrap();
        let entries = list(Kind::Mod, &root.join("mods")).unwrap();
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].name, "a.jar");
        assert!(entries[0].enabled);
        assert_eq!(entries[1].name, "b.jar");
        assert!(!entries[1].enabled);

        set_enabled(&[entries[0].path.clone()], false).unwrap();
        assert!(root.join("mods/a.jar.disabled").is_file());
        set_enabled(&[root.join("mods/a.jar.disabled")], true).unwrap();
        assert!(root.join("mods/a.jar").is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn imports_files_without_overwriting_and_unpacks_saves() {
        let root = root();
        let saves = root.join("saves");
        fs::create_dir_all(&root).unwrap();
        let source = root.join("world.zip");
        {
            let mut zip = zip::ZipWriter::new(fs::File::create(&source).unwrap());
            zip.start_file("world/level.dat", zip::write::SimpleFileOptions::default())
                .unwrap();
            zip.write_all(b"level").unwrap();
            zip.finish().unwrap();
        }
        import(std::slice::from_ref(&source), Kind::Save, &saves).unwrap();
        assert!(saves.join("world/level.dat").is_file());
        assert_eq!(list(Kind::Save, &saves).unwrap().len(), 1);

        let mods = root.join("mods");
        let mod_source = root.join("a.jar");
        fs::write(&mod_source, b"a").unwrap();
        import(std::slice::from_ref(&mod_source), Kind::Mod, &mods).unwrap();
        fs::write(mods.join("a.jar"), b"conflict").unwrap();
        import(&[mod_source], Kind::Mod, &mods).unwrap();
        assert!(mods.join("a (2).jar").is_file());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn save_names_cannot_escape_the_saves_folder() {
        let root = root();
        assert!(directory(Kind::Datapack, &root, Some("../escape")).is_err());
        assert!(directory(Kind::Datapack, &root, None).is_err());
        assert_eq!(
            directory(Kind::Datapack, &root, Some("world")).unwrap(),
            root.join("saves/world/datapacks")
        );
    }
}
