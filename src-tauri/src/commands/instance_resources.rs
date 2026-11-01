use crate::{
    app::config::ConfigManager,
    io::files::open_directory,
    minecraft::java::{self, preference::current_instance},
};

/// Re-download missing or broken instance files (CE "补全文件").
#[tauri::command]
pub async fn repair_instance_files(instance_directory: std::path::PathBuf) -> Result<(), String> {
    let info = java::preference::prepare(instance_directory.clone()).await?;
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
                crate::minecraft::instance::isolation::directory(&state, instance).to_path_buf();
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
        Some(arch) => crate::minecraft::version::McRuleContext::for_java(&arch)
            .map_err(|error| error.to_string())?,
        None => crate::minecraft::version::McRuleContext::current(),
    };
    let downloader = crate::io::download::Downloader::with_bases(
        ConfigManager::instance().api_client.api_bases_async().await,
    );
    if vanilla && details.downloads.client.validate().is_ok() {
        downloader
            .download(&details.downloads.client, &jar_path)
            .await
            .map_err(|error| format!("Cannot repair client JAR: {error}"))?;
    }
    crate::minecraft::installer::prepare_launch_libraries(
        &details,
        &global_dir,
        &directory,
        &context,
        &downloader,
        true,
    )
    .await
    .map_err(|error| format!("{error:#}"))?;
    crate::minecraft::installer::prepare_legacy_assets(
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
fn instance_game_directory(
    instance_directory: &std::path::Path,
) -> Result<std::path::PathBuf, String> {
    ConfigManager::instance().ensure_instance_isolation(instance_directory)?;
    let state = ConfigManager::instance().app_state.blocking_lock();
    let instance =
        current_instance(&state, instance_directory).map_err(|error| error.to_string())?;
    Ok(crate::minecraft::instance::isolation::directory(&state, instance).to_path_buf())
}

#[tauri::command]
pub async fn instance_resource_destination(
    instance_directory: std::path::PathBuf,
    kind: String,
    filename: String,
) -> Result<std::path::PathBuf, String> {
    tauri::async_runtime::spawn_blocking(move || {
        crate::io::files::validate_file_name(&filename).map_err(|e| e.to_string())?;
        let kind = crate::minecraft::instance::files::Kind::parse(&kind)?;
        if !matches!(
            kind,
            crate::minecraft::instance::files::Kind::Mod
                | crate::minecraft::instance::files::Kind::ResourcePack
                | crate::minecraft::instance::files::Kind::Shader
                | crate::minecraft::instance::files::Kind::Save
        ) {
            return Err("Select a destination for this resource type".into());
        }
        let root = instance_game_directory(&instance_directory)?;
        let folder = crate::minecraft::instance::files::directory(kind, &root, None)?;
        std::fs::create_dir_all(&folder).map_err(|e| e.to_string())?;
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

#[tauri::command]
pub async fn list_instance_resources(
    instance_directory: std::path::PathBuf,
    kind: String,
    save: Option<String>,
) -> Result<Vec<crate::minecraft::instance::files::Entry>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let kind = crate::minecraft::instance::files::Kind::parse(&kind)?;
        let game_directory = instance_game_directory(&instance_directory)?;
        let directory =
            crate::minecraft::instance::files::directory(kind, &game_directory, save.as_deref())?;
        let mut entries = crate::minecraft::instance::files::list(kind, &directory)?;
        for entry in &mut entries {
            let picture = match kind {
                crate::minecraft::instance::files::Kind::Screenshot => Some(entry.path.as_path()),
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
        let kind = crate::minecraft::instance::files::Kind::parse(&kind)?;
        let game_directory = instance_game_directory(&instance_directory)?;
        let directory =
            crate::minecraft::instance::files::directory(kind, &game_directory, save.as_deref())?;
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
        crate::minecraft::instance::files::set_enabled(&paths, enabled)
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
        crate::minecraft::instance::files::delete(&paths)
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
        let kind = crate::minecraft::instance::files::Kind::parse(&kind)?;
        let game_directory = instance_game_directory(&instance_directory)?;
        let directory =
            crate::minecraft::instance::files::directory(kind, &game_directory, save.as_deref())?;
        crate::minecraft::instance::files::import(&sources, kind, &directory)
    })
    .await
    .map_err(|error| error.to_string())?
}
