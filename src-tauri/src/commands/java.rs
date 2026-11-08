use crate::config::java_preference::{InstanceJavaSettings, JavaLaunchInfo, JavaPreference};
use crate::config::{current_instance, manager::ConfigManager, states::AppState};
use pcl_core::{
    io::files::open_directory,
    minecraft::java::{self, JavaEntry, JavaSource},
};
use std::sync::Arc;
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::Mutex;

#[tauri::command]
pub async fn add_java(app: AppHandle) -> Result<Option<JavaEntry>, String> {
    let picked =
        tauri::async_runtime::spawn_blocking(move || app.dialog().file().blocking_pick_file())
            .await
            .map_err(|e| e.to_string())?;
    let Some(file) = picked else {
        return Ok(None);
    };
    let path = file.into_path().map_err(|e| e.to_string())?;
    let mut runtime = tauri::async_runtime::spawn_blocking(move || JavaEntry::probe(&path))
        .await
        .map_err(|e| e.to_string())?
        .map_err(|e| e.to_string())?;
    runtime.source = JavaSource::ManualAdded;
    let runtime = tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            if let Some(existing) = state
                .java_runtimes
                .iter_mut()
                .find(|j| j.java_exe == runtime.java_exe)
            {
                existing.source = JavaSource::ManualAdded;
                runtime = existing.clone();
            } else {
                state.java_runtimes.push(runtime.clone());
                java::sort_list(&mut state.java_runtimes);
            }
            Ok(runtime)
        })
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(Some(runtime))
}

#[tauri::command]
pub async fn get_java_list(
    state: State<'_, Arc<Mutex<AppState>>>,
) -> Result<Vec<JavaEntry>, String> {
    Ok(state.lock().await.java_runtimes.clone())
}

#[tauri::command]
pub async fn refresh_java_list() -> Result<Vec<JavaEntry>, String> {
    crate::config::java::refresh(true).await
}

#[derive(serde::Serialize)]
pub struct JavaSettingsInfo {
    global_java: Option<std::path::PathBuf>,
    instance: InstanceJavaSettings,
    has_instance_settings: bool,
    global_is_64_bit: bool,
    instance_is_64_bit: bool,
}
#[tauri::command]
pub async fn get_java_settings(
    instance_directory: Option<std::path::PathBuf>,
) -> Result<JavaSettingsInfo, String> {
    let state = ConfigManager::instance().app_state.lock().await;
    let has_instance_settings = instance_directory
        .as_ref()
        .is_some_and(|d| state.java_preferences.contains_key(d));
    let instance = if let Some(directory) = instance_directory {
        current_instance(&state, &directory).map_err(|e| e.to_string())?;
        state
            .java_preferences
            .get(&directory)
            .cloned()
            .unwrap_or_default()
    } else {
        InstanceJavaSettings::default()
    };
    let runtimes = state.java_runtimes.clone();
    let global_java = state.launcher_setup.selected_java.clone();
    drop(state);
    tauri::async_runtime::spawn_blocking(move || {
        let global_is_64_bit = crate::config::java_preference::is_game_set_64_bit(
            &runtimes,
            global_java.as_deref(),
            &JavaPreference::Global,
        );
        let instance_is_64_bit = crate::config::java_preference::is_game_set_64_bit(
            &runtimes,
            global_java.as_deref(),
            &instance.preference,
        );
        JavaSettingsInfo {
            global_java,
            instance,
            has_instance_settings,
            global_is_64_bit,
            instance_is_64_bit,
        }
    })
    .await
    .map_err(|e| e.to_string())
}
#[tauri::command]
pub async fn set_global_java(java_exe: Option<std::path::PathBuf>) -> Result<(), String> {
    let selected = if let Some(path) = java_exe {
        let actual = tauri::async_runtime::spawn_blocking(move || JavaEntry::probe(&path))
            .await
            .map_err(|e| e.to_string())?
            .map_err(|e| e.to_string())?;
        Some(actual.java_exe().to_owned())
    } else {
        None
    };
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            if let Some(path) = &selected {
                let saved = state
                    .java_runtimes
                    .iter()
                    .find(|j| &j.java_exe == path)
                    .ok_or("Java is unavailable; refresh the list")?;
                if !saved.is_enabled {
                    return Err("Enable this Java before selecting it".into());
                }
            }
            state.launcher_setup.selected_java = selected;
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn set_java_enabled(java_exe: std::path::PathBuf, enabled: bool) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            if !enabled && state.launcher_setup.selected_java.as_ref() == Some(&java_exe) {
                return Err("Deselect this Java as the global default before disabling it".into());
            }
            let runtime = state
                .java_runtimes
                .iter_mut()
                .find(|j| j.java_exe == java_exe)
                .ok_or("Java is unavailable; refresh the list")?;
            runtime.is_enabled = enabled;
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn set_instance_java_settings(
    instance_directory: std::path::PathBuf,
    settings: InstanceJavaSettings,
) -> Result<(), String> {
    let validated = settings.clone();
    tauri::async_runtime::spawn_blocking(move || {
        crate::config::java_preference::validate_preference(&validated.preference)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            current_instance(state, &instance_directory).map_err(|e| e.to_string())?;
            state.java_preferences.insert(instance_directory, settings);
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn select_relative_java(app: AppHandle) -> Result<Option<JavaPreference>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let root = crate::config::paths::launcher_directory().map_err(|e| e.to_string())?;
        let Some(file) = app
            .dialog()
            .file()
            .set_directory(&root)
            .blocking_pick_file()
        else {
            return Ok(None);
        };
        let path = file
            .into_path()
            .map_err(|e| e.to_string())?
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let root = root.canonicalize().map_err(|e| e.to_string())?;
        let relative = path
            .strip_prefix(&root)
            .map_err(|_| "Java path is outside the launcher directory")?
            .to_owned();
        crate::config::java_preference::relative_java(&root, &relative)
            .map_err(|e| e.to_string())?;
        JavaEntry::probe(&path).map_err(|e| e.to_string())?;
        Ok(Some(JavaPreference::Relative {
            relative_path: relative,
        }))
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn open_java_folder(java_exe: std::path::PathBuf) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let runtime = JavaEntry::probe(&java_exe).map_err(|e| e.to_string())?;
        open_directory(runtime.java_folder())
    })
    .await
    .map_err(|e| e.to_string())?
}
#[tauri::command]
pub async fn prepare_game_java(
    instance_directory: std::path::PathBuf,
) -> Result<JavaLaunchInfo, String> {
    crate::config::java::prepare(instance_directory).await
}

static JAVA_INSTALL: Mutex<Option<tokio::task::AbortHandle>> = Mutex::const_new(None);
#[tauri::command]
pub async fn download_game_java(
    instance_directory: std::path::PathBuf,
    progress: tauri::ipc::Channel<java::install::JavaInstallProgress>,
) -> Result<JavaEntry, String> {
    let mut active = JAVA_INSTALL.lock().await;
    if active.is_some() {
        return Err("A Java download is already running".into());
    }
    let info = crate::config::java::prepare(instance_directory).await?;
    let config = ConfigManager::instance();
    let downloader = config.api_client.downloader();
    let task = tauri::async_runtime::spawn(java::install::install(
        info.requirements,
        config.java_runtime_dir(),
        downloader,
        move |update| {
            let _ = progress.send(update);
        },
    ));
    *active = Some(task.inner().abort_handle());
    drop(active);
    let result = task
        .await
        .map_err(|e| e.to_string())
        .and_then(|r| r.map_err(|e| e.to_string()));
    *JAVA_INSTALL.lock().await = None;
    let runtime = result?;
    let runtime = tauri::async_runtime::spawn_blocking(move || {
        config.update(|state| {
            state.java_runtimes.push(runtime.clone());
            java::sort_list(&mut state.java_runtimes);
            Ok(runtime)
        })
    })
    .await
    .map_err(|e| e.to_string())??;
    Ok(runtime)
}
#[tauri::command]
pub async fn cancel_java_download() {
    if let Some(task) = JAVA_INSTALL.lock().await.as_ref() {
        task.abort();
    }
}
