use crate::config::{current_instance, manager::ConfigManager};
use pcl_core::io::files::open_directory;

/// Re-download missing or broken instance files (CE "补全文件").
#[tauri::command]
pub async fn repair_instance_files(instance_directory: std::path::PathBuf) -> Result<(), String> {
    let info = crate::config::java::prepare(instance_directory.clone()).await?;
    let arch = info
        .runtime
        .as_ref()
        .map(|runtime| runtime.architecture.clone());
    let (details, global_dir, directory, game_directory, jar_path, vanilla) =
        tauri::async_runtime::spawn_blocking(move || {
            ConfigManager::instance().ensure_instance_isolation(&instance_directory)?;
            let state = ConfigManager::instance().app_state.blocking_lock();
            let instance =
                current_instance(&state, &instance_directory).map_err(|e| e.to_string())?;
            let details = instance
                .read_version_json()
                .map_err(|error| error.to_string())?;
            let game_directory =
                crate::config::isolation::directory(&state.instance_isolation, instance)
                    .to_path_buf();
            // Loader instances share the parent's client JAR; only vanilla owns `jar_path`.
            let vanilla = instance
                .plugins_version
                .keys()
                .all(|plugin| plugin == "vanilla");
            Ok::<_, String>((
                details,
                instance.global_dir.clone(),
                instance.directory.clone(),
                game_directory,
                instance.jar_path.clone(),
                vanilla,
            ))
        })
        .await
        .map_err(|error| error.to_string())??;
    let context = match arch {
        Some(arch) => pcl_core::minecraft::version::McRuleContext::for_java(&arch)
            .map_err(|error| error.to_string())?,
        None => pcl_core::minecraft::version::McRuleContext::current(),
    };
    let downloader = ConfigManager::instance().api_client.downloader().await;
    if vanilla && details.downloads.client.validate().is_ok() {
        downloader
            .download(&details.downloads.client, &jar_path)
            .await
            .map_err(|error| format!("Cannot repair client JAR: {error}"))?;
    }
    pcl_core::minecraft::installer::prepare_launch_libraries(
        &details,
        &global_dir,
        &directory,
        &context,
        &downloader,
        true,
    )
    .await
    .map_err(|error| format!("{error:#}"))?;
    pcl_core::minecraft::installer::prepare_legacy_assets(
        &details,
        &global_dir,
        &game_directory,
        &downloader,
        true,
    )
    .await
    .map_err(|error| format!("{error:#}"))?;
    Ok(())
}
/// Game directory that owns the instance resource folders (isolation aware).
pub(crate) fn instance_game_directory(
    instance_directory: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    ConfigManager::instance().ensure_instance_isolation(instance_directory)?;
    let state = ConfigManager::instance().app_state.blocking_lock();
    let instance =
        current_instance(&state, instance_directory).map_err(|error| error.to_string())?;
    Ok(crate::config::isolation::directory(&state.instance_isolation, instance).to_path_buf())
}

#[tauri::command]
pub async fn instance_resource_destination(
    instance_directory: std::path::PathBuf,
    kind: String,
    filename: String,
    save: Option<String>,
) -> Result<std::path::PathBuf, String> {
    tauri::async_runtime::spawn_blocking(move || {
        pcl_core::io::files::validate_file_name(&filename).map_err(|e| e.to_string())?;
        let kind = pcl_core::minecraft::instance::files::Kind::parse(&kind)?;
        if !matches!(
            kind,
            pcl_core::minecraft::instance::files::Kind::Mod
                | pcl_core::minecraft::instance::files::Kind::ResourcePack
                | pcl_core::minecraft::instance::files::Kind::Shader
                | pcl_core::minecraft::instance::files::Kind::Save
                | pcl_core::minecraft::instance::files::Kind::Datapack
        ) {
            return Err("Select a destination for this resource type".into());
        }
        let root = instance_game_directory(&instance_directory)?;
        let folder = if kind == pcl_core::minecraft::instance::files::Kind::Datapack {
            datapack_destination(
                &root,
                save.as_deref()
                    .ok_or("A save name is required for datapacks")?,
            )?
        } else {
            let folder = pcl_core::minecraft::instance::files::directory(kind, &root, None)?;
            std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
            folder
        };
        Ok(folder.join(filename))
    })
    .await
    .map_err(|e| e.to_string())?
}

/// Resource mutations must stay inside the instance game directory.
fn ensure_inside_instance(root: &std::path::Path, path: &std::path::Path) -> Result<(), String> {
    let root = root.canonicalize().map_err(|error| error.to_string())?;
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    if !canonical.starts_with(&root) {
        return Err("Resource path is outside the instance directory".into());
    }
    Ok(())
}

fn save_directory(
    instance_directory: &std::path::Path,
    save: &str,
) -> Result<std::path::PathBuf, String> {
    let game = instance_game_directory(instance_directory)?;
    save_in_game(&game, save)
}

fn save_in_game(game: &std::path::Path, save: &str) -> Result<std::path::PathBuf, String> {
    use pcl_core::minecraft::instance::files::{Kind, directory};
    let folder = directory(Kind::Datapack, game, Some(save))?
        .parent()
        .unwrap()
        .to_path_buf();
    let saves = directory(Kind::Save, game, None)?;
    ensure_inside_instance(game, &saves)?;
    let saves = saves.canonicalize().map_err(|e| e.to_string())?;
    let folder = folder.canonicalize().map_err(|e| e.to_string())?;
    if folder.parent() != Some(saves.as_path()) || !folder.is_dir() {
        return Err("Save is outside the saves directory".into());
    }
    Ok(folder)
}

fn datapack_destination(game: &std::path::Path, save: &str) -> Result<std::path::PathBuf, String> {
    let save = save_in_game(game, save)?;
    let folder = save.join("datapacks");
    match std::fs::create_dir(&folder) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.to_string()),
    }
    let metadata = std::fs::symlink_metadata(&folder).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.file_type().is_symlink() {
        return Err("Datapack folder must be a directory inside the save".into());
    }
    Ok(folder)
}

#[tauri::command]
pub async fn get_save_info(
    instance_directory: std::path::PathBuf,
    save: String,
) -> Result<pcl_core::minecraft::saves::SaveInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        pcl_core::minecraft::saves::read(&save_directory(&instance_directory, &save)?)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn update_save_info(
    instance_directory: std::path::PathBuf,
    save: String,
    changes: pcl_core::minecraft::saves::SaveChanges,
) -> Result<pcl_core::minecraft::saves::SaveInfo, String> {
    tauri::async_runtime::spawn_blocking(move || {
        pcl_core::minecraft::saves::apply(&save_directory(&instance_directory, &save)?, changes)
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn list_instance_resources(
    instance_directory: std::path::PathBuf,
    kind: String,
    save: Option<String>,
) -> Result<Vec<pcl_core::minecraft::instance::files::Entry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let kind = pcl_core::minecraft::instance::files::Kind::parse(&kind)?;
        let game_directory = instance_game_directory(&instance_directory)?;
        let directory = pcl_core::minecraft::instance::files::directory(
            kind,
            &game_directory,
            save.as_deref(),
        )?;
        let mut entries = pcl_core::minecraft::instance::files::list(kind, &directory)?;
        for entry in &mut entries {
            let picture = match kind {
                pcl_core::minecraft::instance::files::Kind::Screenshot => {
                    Some(entry.path.as_path())
                }
                _ => entry.icon.as_deref(),
            };
            entry.token = picture.map(crate::protocol::register);
        }
        Ok(entries)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn open_instance_resource_folder(
    instance_directory: std::path::PathBuf,
    kind: String,
    save: Option<String>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let kind = pcl_core::minecraft::instance::files::Kind::parse(&kind)?;
        let game_directory = instance_game_directory(&instance_directory)?;
        let directory = pcl_core::minecraft::instance::files::directory(
            kind,
            &game_directory,
            save.as_deref(),
        )?;
        std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        open_directory(&directory)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn open_instance_path(
    instance_directory: std::path::PathBuf,
    path: std::path::PathBuf,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let game_directory = instance_game_directory(&instance_directory)?;
        ensure_inside_instance(&game_directory, &path)?;
        open_directory(&path)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_instance_resources_enabled(
    instance_directory: std::path::PathBuf,
    paths: Vec<std::path::PathBuf>,
    enabled: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let game_directory = instance_game_directory(&instance_directory)?;
        for path in &paths {
            ensure_inside_instance(&game_directory, path)?;
        }
        pcl_core::minecraft::instance::files::set_enabled(&paths, enabled)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn delete_instance_resources(
    instance_directory: std::path::PathBuf,
    paths: Vec<std::path::PathBuf>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let game_directory = instance_game_directory(&instance_directory)?;
        for path in &paths {
            ensure_inside_instance(&game_directory, path)?;
        }
        pcl_core::minecraft::instance::files::delete(&paths)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn import_instance_resources(
    instance_directory: std::path::PathBuf,
    kind: String,
    save: Option<String>,
    sources: Vec<std::path::PathBuf>,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let kind = pcl_core::minecraft::instance::files::Kind::parse(&kind)?;
        let game_directory = instance_game_directory(&instance_directory)?;
        let directory = pcl_core::minecraft::instance::files::directory(
            kind,
            &game_directory,
            save.as_deref(),
        )?;
        pcl_core::minecraft::instance::files::import(&sources, kind, &directory)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn datapack_destination_requires_an_existing_save() {
        let root = tempfile::tempdir().unwrap();
        let world = root.path().join("saves/world");
        std::fs::create_dir_all(&world).unwrap();
        let destination = datapack_destination(root.path(), "world").unwrap();
        assert_eq!(destination, world.canonicalize().unwrap().join("datapacks"));
        assert!(destination.is_dir());
        assert!(datapack_destination(root.path(), "../escape").is_err());
        assert!(datapack_destination(root.path(), "missing").is_err());
        assert!(!root.path().join("saves/missing").exists());
    }

    #[cfg(unix)]
    #[test]
    fn datapack_destination_rejects_linked_folders() {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir().unwrap();
        let outside = tempfile::tempdir().unwrap();
        let world = root.path().join("saves/world");
        std::fs::create_dir_all(&world).unwrap();
        symlink(outside.path(), world.join("datapacks")).unwrap();
        assert!(datapack_destination(root.path(), "world").is_err());
        assert!(std::fs::read_dir(outside.path()).unwrap().next().is_none());
    }
}
