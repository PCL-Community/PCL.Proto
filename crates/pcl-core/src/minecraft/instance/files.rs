use std::{
    fs,
    io::Read,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use base64::{Engine as _, engine::general_purpose::STANDARD};
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
    pub created: u64,
    pub enabled: bool,
    pub icon: Option<PathBuf>,
    /// Filled by the command layer so the webview can load the picture via a scoped protocol.
    pub token: Option<String>,
    pub display_name: Option<String>,
    pub version: Option<String>,
    pub description: Option<String>,
    pub icon_data: Option<String>,
}

#[derive(Default)]
struct ModDetails {
    name: Option<String>,
    version: Option<String>,
    description: Option<String>,
    icon: Option<String>,
}

fn archive_bytes(
    archive: &mut zip::ZipArchive<fs::File>,
    name: &str,
    limit: u64,
) -> Option<Vec<u8>> {
    let mut file = archive.by_name(name).ok()?;
    if file.size() > limit {
        return None;
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).ok()?;
    Some(bytes)
}

fn mod_details(path: &Path) -> ModDetails {
    let Some(mut archive) = fs::File::open(path)
        .ok()
        .and_then(|file| zip::ZipArchive::new(file).ok())
    else {
        return ModDetails::default();
    };
    let json = ["fabric.mod.json", "quilt.mod.json", "mcmod.info"]
        .iter()
        .find_map(|name| {
            let bytes = archive_bytes(&mut archive, name, 1024 * 1024)?;
            let mut value: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
            if *name == "quilt.mod.json" {
                value = value.get("quilt_loader")?.clone();
            }
            if let Some(array) = value.as_array() {
                value = array.first()?.clone();
            }
            Some(value)
        });
    let mut details = ModDetails::default();
    if let Some(value) = json {
        let metadata = value.get("metadata").unwrap_or(&value);
        let field = |v: &serde_json::Value, key: &str| {
            v.get(key)
                .and_then(|v| v.as_str())
                .filter(|v| !v.is_empty())
                .map(str::to_owned)
        };
        details.name = field(metadata, "name").or_else(|| field(&value, "id"));
        details.version = field(&value, "version");
        details.description = field(metadata, "description");
        details.icon = field(metadata, "icon")
            .or_else(|| {
                metadata
                    .get("icon")?
                    .as_object()?
                    .values()
                    .filter_map(|v| v.as_str())
                    .next_back()
                    .map(str::to_owned)
            })
            .or_else(|| field(metadata, "logoFile"));
    } else {
        for name in ["META-INF/neoforge.mods.toml", "META-INF/mods.toml"] {
            let Some(bytes) = archive_bytes(&mut archive, name, 1024 * 1024) else {
                continue;
            };
            let Some(value) = std::str::from_utf8(&bytes)
                .ok()
                .and_then(|s| toml::from_str::<toml::Value>(s).ok())
            else {
                continue;
            };
            let Some(info) = value
                .get("mods")
                .and_then(|v| v.as_array())
                .and_then(|v| v.first())
            else {
                continue;
            };
            let field = |key: &str| {
                info.get(key)
                    .and_then(|v| v.as_str())
                    .filter(|v| !v.is_empty())
                    .map(str::to_owned)
            };
            details.name = field("displayName").or_else(|| field("modId"));
            details.version = field("version").filter(|v| !v.contains("${"));
            details.description = field("description");
            details.icon = field("logoFile");
            break;
        }
    }
    details.icon = details.icon.and_then(|name| {
        let mime = if name.to_ascii_lowercase().ends_with(".png") {
            "image/png"
        } else if name.to_ascii_lowercase().ends_with(".jpg")
            || name.to_ascii_lowercase().ends_with(".jpeg")
        {
            "image/jpeg"
        } else {
            return None;
        };
        let bytes = archive_bytes(&mut archive, &name, 2 * 1024 * 1024)?;
        Some(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
    });
    details
}

fn pack_details(path: &Path) -> ModDetails {
    let mut details = ModDetails::default();
    let mut archive = fs::File::open(path)
        .ok()
        .and_then(|file| zip::ZipArchive::new(file).ok());
    let mut read = |name: &str, limit: u64| {
        if path.is_dir() {
            let target = path.join(name);
            if fs::symlink_metadata(&target).ok()?.file_type().is_symlink() {
                return None;
            }
            let file = fs::File::open(target).ok()?;
            if file.metadata().ok()?.len() > limit {
                return None;
            }
            let mut bytes = Vec::new();
            file.take(limit + 1).read_to_end(&mut bytes).ok()?;
            (bytes.len() as u64 <= limit).then_some(bytes)
        } else {
            archive_bytes(archive.as_mut()?, name, limit)
        }
    };
    if let Some(bytes) = read("pack.mcmeta", 1024 * 1024)
        && let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes)
    {
        details.description = value
            .get("pack")
            .and_then(|pack| pack.get("description"))
            .and_then(|description| {
                description
                    .as_str()
                    .or_else(|| description.get("text").and_then(|text| text.as_str()))
            })
            .map(str::to_owned);
    }
    details.icon = read("pack.png", 2 * 1024 * 1024)
        .map(|bytes| format!("data:image/png;base64,{}", STANDARD.encode(bytes)));
    details
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
            Kind::Save => {
                metadata.is_dir()
                    && (path.join("level.dat").is_file() || path.join("level.dat_old").is_file())
            }
            Kind::Datapack => metadata.is_dir() || (metadata.is_file() && base.ends_with(".zip")),
        };
        if !matched {
            continue;
        }
        let name = match kind {
            Kind::Save => raw_name.to_owned(),
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
        let details = match kind {
            Kind::Mod if metadata.is_file() => mod_details(&path),
            Kind::ResourcePack | Kind::Datapack => pack_details(&path),
            _ => ModDetails::default(),
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
            created: metadata
                .created()
                .ok()
                .and_then(|time| time.duration_since(UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0),
            enabled: !disabled,
            icon,
            token: None,
            display_name: details.name,
            version: details.version,
            description: details.description,
            icon_data: details.icon,
        });
    }
    entries.sort_by_key(|entry| entry.name.to_lowercase());
    Ok(entries)
}

fn strip_suffix<'a>(name: &'a str, suffix: &str) -> Option<&'a str> {
    let start = name.len().checked_sub(suffix.len())?;
    (start > 0 && name.get(start..)?.eq_ignore_ascii_case(suffix))
        .then(|| name.get(..start))
        .flatten()
}

/// Enable or disable files by toggling the CE-compatible `.disabled` suffix.
pub fn set_enabled(paths: &[PathBuf], enabled: bool) -> Result<(), String> {
    for path in paths {
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            return Err("Invalid resource path".into());
        };
        let lower = name.to_ascii_lowercase();
        let disabled = lower.ends_with(".disabled") || lower.ends_with(".old");
        if enabled == !disabled {
            continue;
        }
        let target = if enabled {
            path.with_file_name(
                strip_suffix(name, ".disabled")
                    .or_else(|| strip_suffix(name, ".old"))
                    .ok_or("Resource is not disabled")?,
            )
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
        // 存档 zip 由用户提供，头里声明的大小不可信，按实际写入字节限 512 MiB。
        crate::io::files::copy_entry_bounded(&mut file, &mut output, 512 * 1024 * 1024)
            .map_err(|e| e.to_string())?;
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
    use std::io::Write;

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
    fn reads_mod_metadata_and_icons_from_archives() {
        let root = root();
        fs::create_dir_all(&root).unwrap();
        let path = root.join("fabric.jar");
        let mut zip = zip::ZipWriter::new(fs::File::create(&path).unwrap());
        zip.start_file("fabric.mod.json", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(
            br#"{"name":"Example","version":"1.2","description":"Hello","icon":"assets/icon.png"}"#,
        )
        .unwrap();
        zip.start_file("assets/icon.png", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"image").unwrap();
        zip.finish().unwrap();
        let entries = list(Kind::Mod, &root).unwrap();
        assert_eq!(entries[0].display_name.as_deref(), Some("Example"));
        assert_eq!(entries[0].version.as_deref(), Some("1.2"));
        assert_eq!(entries[0].description.as_deref(), Some("Hello"));
        assert!(
            entries[0]
                .icon_data
                .as_deref()
                .unwrap()
                .starts_with("data:image/png;base64,")
        );
        let forge_path = root.join("forge.jar");
        let mut zip = zip::ZipWriter::new(fs::File::create(&forge_path).unwrap());
        zip.start_file(
            "META-INF/mods.toml",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
        zip.write_all(b"[[mods]]\nmodId = 'example'\ndisplayName = 'Forge Example'\nversion = '2.0'\ndescription = 'Forge description'\n").unwrap();
        zip.finish().unwrap();
        let details = mod_details(&forge_path);
        assert_eq!(details.name.as_deref(), Some("Forge Example"));
        assert_eq!(details.version.as_deref(), Some("2.0"));
        assert_eq!(details.description.as_deref(), Some("Forge description"));
        let pack = root.join("pack.zip");
        let mut zip = zip::ZipWriter::new(fs::File::create(&pack).unwrap());
        zip.start_file("pack.mcmeta", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(br#"{"pack":{"description":"Example pack"}}"#)
            .unwrap();
        zip.start_file("pack.png", zip::write::SimpleFileOptions::default())
            .unwrap();
        zip.write_all(b"image").unwrap();
        zip.finish().unwrap();
        let details = pack_details(&pack);
        assert_eq!(details.description.as_deref(), Some("Example pack"));
        assert!(details.icon.unwrap().starts_with("data:image/png;base64,"));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn lists_and_toggles_unicode_resource_names() {
        let root = root();
        let saves = root.join("saves");
        let mods = root.join("mods");
        fs::create_dir_all(saves.join("我的世界")).unwrap();
        fs::write(saves.join("我的世界/level.dat"), b"level").unwrap();
        fs::create_dir_all(&mods).unwrap();
        fs::write(mods.join("世界.jar.DISABLED"), b"mod").unwrap();

        let worlds = list(Kind::Save, &saves).unwrap();
        assert_eq!(worlds[0].name, "我的世界");
        let entries = list(Kind::Mod, &mods).unwrap();
        assert_eq!(entries[0].name, "世界.jar");
        assert!(!entries[0].enabled);
        set_enabled(&[entries[0].path.clone()], true).unwrap();
        assert!(mods.join("世界.jar").is_file());
        set_enabled(&[mods.join("世界.jar")], false).unwrap();
        assert!(mods.join("世界.jar.disabled").is_file());
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
