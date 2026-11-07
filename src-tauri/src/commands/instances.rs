use crate::config::{current_instance, manager::ConfigManager, states::AppState};
use pcl_core::{
    io::files::open_directory,
    minecraft::instance::{McInstance, McInstanceFolder},
};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn get_isolation_default() -> crate::config::isolation::IsolationDefault {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .isolation_default
}

#[tauri::command]
pub async fn set_isolation_default(
    value: crate::config::isolation::IsolationDefault,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance()
            .update(|state| crate::config::isolation::set_default(state, value))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_instance_isolation(
    instance_directory: std::path::PathBuf,
) -> Result<bool, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().ensure_instance_isolation(&instance_directory)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_instance_isolation(
    instance_directory: std::path::PathBuf,
    value: bool,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            current_instance(state, &instance_directory).map_err(|error| error.to_string())?;
            state.instance_isolation.insert(instance_directory, value);
            Ok(())
        })
    })
    .await
    .map_err(|error| error.to_string())?
}
#[tauri::command]
pub async fn open_instance_folder(
    instance_directory: std::path::PathBuf,
    kind: McInstanceFolder,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().ensure_instance_isolation(&instance_directory)?;
        let state = ConfigManager::instance().app_state.blocking_lock();
        let instance =
            current_instance(&state, &instance_directory).map_err(|error| error.to_string())?;
        let game_directory =
            crate::config::isolation::directory(&state.instance_isolation, instance);
        let directory = pcl_core::minecraft::instance::prepare_shortcut_directory(
            instance,
            game_directory,
            kind,
        )
        .map_err(|error| error.to_string())?;
        open_directory(&directory)
    })
    .await
    .map_err(|error| error.to_string())?
}
#[tauri::command(rename_all = "snake_case")]
pub async fn instance_folder_action(
    repository_index: usize,
    instance_name: String,
    instance_directory: std::path::PathBuf,
    delete: bool,
) -> Result<(), String> {
    pcl_core::io::files::validate_file_name(&instance_name).map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        let manager = ConfigManager::instance();
        let repo = manager
            .app_state
            .blocking_lock()
            .repositories
            .get(repository_index)
            .cloned()
            .ok_or("Minecraft folder index is out of range")?;
        let instance = repo
            .game_instance_named(&instance_name)
            .ok_or("Minecraft instance was not found")?;
        if instance.directory != instance_directory {
            return Err("Minecraft folder changed; try again".into());
        }
        let versions = repo
            .path
            .join("versions")
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let path = instance
            .directory
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if path.parent() != Some(versions.as_path())
            || std::fs::symlink_metadata(&instance.directory)
                .map_err(|e| e.to_string())?
                .file_type()
                .is_symlink()
        {
            return Err("Instance directory is not a direct, canonical child of versions".into());
        }
        if !delete {
            return open_directory(&path);
        }
        trash::delete(&path).map_err(|e| e.to_string())?;
        manager.update(|state| {
            if state
                .active_game_instance
                .as_ref()
                .is_some_and(|active| active.directory == instance.directory)
            {
                state.active_game_instance = None;
            }
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn get_instance_description(
    instance_directory: std::path::PathBuf,
) -> Result<String, String> {
    let state = ConfigManager::instance().app_state.lock().await;
    current_instance(&state, &instance_directory).map_err(|error| error.to_string())?;
    Ok(state
        .instance_descriptions
        .get(&instance_directory)
        .cloned()
        .unwrap_or_default())
}

#[tauri::command]
pub async fn set_instance_description(
    instance_directory: std::path::PathBuf,
    description: String,
) -> Result<String, String> {
    if description.len() > 4096 || description.contains('\0') {
        return Err("Invalid instance description".into());
    }
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            current_instance(state, &instance_directory).map_err(|e| e.to_string())?;
            if description.trim().is_empty() {
                state.instance_descriptions.remove(&instance_directory);
            } else {
                state
                    .instance_descriptions
                    .insert(instance_directory.clone(), description.clone());
            }
            Ok(description)
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn rename_instance(
    instance_directory: std::path::PathBuf,
    new_name: String,
) -> Result<McInstance, String> {
    pcl_core::io::files::validate_file_name(&new_name).map_err(|error| error.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        rename_instance_inner(instance_directory, new_name)
    })
    .await
    .map_err(|error| error.to_string())?
}

fn rename_instance_inner(
    instance_directory: std::path::PathBuf,
    new_name: String,
) -> Result<McInstance, String> {
    let manager = ConfigManager::instance();
    let (instance, repo) = {
        let state = manager.app_state.blocking_lock();
        let instance = current_instance(&state, &instance_directory)
            .map_err(|error| error.to_string())?
            .clone();
        let repo = state
            .repositories
            .iter()
            .find(|repo| repo.path == instance.global_dir)
            .cloned()
            .ok_or("Minecraft folder was not found")?;
        (instance, repo)
    };
    let old_name = instance.name.clone();
    if new_name == old_name {
        return Ok(instance);
    }
    let versions = repo
        .path
        .join("versions")
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let old_directory = instance
        .directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if old_directory.parent() != Some(versions.as_path())
        || std::fs::symlink_metadata(&instance.directory)
            .map_err(|error| error.to_string())?
            .file_type()
            .is_symlink()
    {
        return Err("Instance directory is not a direct, canonical child of versions".into());
    }
    let new_directory = repo.path.join("versions").join(&new_name);
    if new_directory.exists() {
        return Err("An instance with that name already exists".into());
    }
    pcl_core::minecraft::instance::rename_instance_files(
        &instance.directory,
        &new_directory,
        &old_name,
        &new_name,
    )?;
    let refreshed = McInstance::from_version_folder(&new_directory, &repo)
        .map_err(|_| "The renamed instance is no longer valid".to_string())?;
    let saved = manager.update(|state| {
        if let Some(value) = state.instance_memory_custom.remove(&instance.directory) {
            state
                .instance_memory_custom
                .insert(new_directory.clone(), value);
        }
        if let Some(value) = state.instance_memory_settings.remove(&instance.directory) {
            state
                .instance_memory_settings
                .insert(new_directory.clone(), value);
        }
        if let Some(value) = state.instance_launch_arguments.remove(&instance.directory) {
            state
                .instance_launch_arguments
                .insert(new_directory.clone(), value);
        }
        if let Some(value) = state.instance_launch_settings.remove(&instance.directory) {
            state
                .instance_launch_settings
                .insert(new_directory.clone(), value);
        }
        if let Some(value) = state.java_preferences.remove(&instance.directory) {
            state.java_preferences.insert(new_directory.clone(), value);
        }
        if let Some(value) = state.instance_isolation.remove(&instance.directory) {
            state
                .instance_isolation
                .insert(new_directory.clone(), value);
        }
        if let Some(description) = state.instance_descriptions.remove(&instance.directory) {
            state
                .instance_descriptions
                .insert(new_directory.clone(), description);
        }
        if state
            .active_game_instance
            .as_ref()
            .is_some_and(|active| active.directory == instance.directory)
        {
            state.active_game_instance = Some(Arc::new(refreshed.clone()));
        }
        Ok(())
    });
    if let Err(error) = saved {
        // Keep the folder and the stored instance keys consistent.
        if let Err(rollback) = std::fs::rename(&new_directory, &instance.directory) {
            log::warn!("Cannot roll back instance rename: {rollback}");
        }
        return Err(error);
    }
    Ok(refreshed)
}
#[tauri::command(rename_all = "snake_case")]
pub async fn get_active_instance(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Option<McInstance>, String> {
    let Some(active_instance) = state.lock().await.active_game_instance.clone() else {
        return Ok(None);
    };
    tauri::async_runtime::spawn_blocking(move || {
        active_instance
            .refresh()
            .unwrap_or_else(|_| (*active_instance).clone())
    })
    .await
    .map(Some)
    .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn select_instance(
    state: State<'_, Arc<Mutex<AppState>>>,
    repository_index: usize,
    instance_name: String,
) -> Result<McInstance, String> {
    pcl_core::io::files::validate_file_name(&instance_name).map_err(|error| error.to_string())?;
    let repo = state
        .lock()
        .await
        .repositories
        .get(repository_index)
        .cloned()
        .ok_or("Minecraft folder index is out of range")?;
    let repo_path = repo.path.clone();
    let instance =
        tauri::async_runtime::spawn_blocking(move || repo.game_instance_named(&instance_name))
            .await
            .map_err(|error| error.to_string())?
            .ok_or("Minecraft instance was not found")?;
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().select_instance(
            repository_index,
            &repo_path,
            instance.clone(),
        )?;
        Ok(instance)
    })
    .await
    .map_err(|error| error.to_string())?
}
