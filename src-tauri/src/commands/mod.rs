use crate::{
    AppState,
    app::config::ConfigManager,
    io::files::open_directory,
    io::net::http::api_client::{self, plugins::McPluginReport},
    minecraft::{
        instance::{McInstance, McInstanceFolder, folder::McFolder},
        java::{
            self, JavaEntry, JavaSource,
            preference::{InstanceJavaSettings, JavaLaunchInfo, JavaPreference, current_instance},
        },
        launch::launcher::LaunchOption,
        mod_loader,
        profile::SafeProfile,
    },
};
use std::{
    future::Future,
    sync::{
        Arc, LazyLock, Mutex as StdMutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tauri::{AppHandle, State};
use tauri_plugin_dialog::DialogExt;
use tokio::sync::{Mutex, watch};

const LAUNCH_CANCELLED: &str = "Launch cancelled";

#[derive(Clone)]
struct LaunchTicket {
    cancelled: Arc<AtomicBool>,
    receiver: watch::Receiver<bool>,
}

struct ActiveLaunch {
    cancelled: Arc<AtomicBool>,
    sender: watch::Sender<bool>,
    session: Option<String>,
    claimed: bool,
}

#[derive(Default)]
struct LaunchController {
    active: Option<ActiveLaunch>,
}

impl LaunchController {
    fn begin(&mut self) -> Result<LaunchTicket, String> {
        if self.active.is_some() {
            return Err("A game is already launching".into());
        }
        let cancelled = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = watch::channel(false);
        self.active = Some(ActiveLaunch {
            cancelled: cancelled.clone(),
            sender,
            session: None,
            claimed: false,
        });
        Ok(LaunchTicket {
            cancelled,
            receiver,
        })
    }

    fn claim(&mut self) -> Result<LaunchTicket, String> {
        let active = self
            .active
            .as_mut()
            .ok_or_else(|| "No launch has been prepared".to_string())?;
        if active.claimed || active.session.is_some() {
            return Err("A game is already launching".into());
        }
        active.claimed = true;
        Ok(LaunchTicket {
            cancelled: active.cancelled.clone(),
            receiver: active.sender.subscribe(),
        })
    }

    fn cancel(&mut self) -> Result<Option<String>, String> {
        let Some(active) = self.active.as_mut() else {
            return Err("No game launch is active".into());
        };
        active.cancelled.store(true, Ordering::Release);
        let _ = active.sender.send(true);
        let session = active.session.clone();
        if session.is_none() {
            self.active = None;
        }
        Ok(session)
    }

    fn clear_pending(&mut self, ticket: &LaunchTicket) {
        if self.active.as_ref().is_some_and(|active| {
            active.session.is_none() && Arc::ptr_eq(&active.cancelled, &ticket.cancelled)
        }) {
            self.active = None;
        }
    }

    fn attach_session(&mut self, ticket: &LaunchTicket, id: String) -> bool {
        let Some(active) = self.active.as_mut() else {
            return false;
        };
        if !Arc::ptr_eq(&active.cancelled, &ticket.cancelled)
            || active.cancelled.load(Ordering::Acquire)
        {
            return false;
        }
        active.session = Some(id);
        true
    }

    fn finish_session(&mut self, id: &str) {
        if self
            .active
            .as_ref()
            .and_then(|active| active.session.as_deref())
            == Some(id)
        {
            self.active = None;
        }
    }
}

static LAUNCH_CONTROLLER: LazyLock<StdMutex<LaunchController>> =
    LazyLock::new(|| StdMutex::new(LaunchController::default()));

fn begin_launch_ticket() -> Result<LaunchTicket, String> {
    LAUNCH_CONTROLLER
        .lock()
        .map_err(|error| error.to_string())?
        .begin()
}

fn claim_launch_ticket() -> Result<LaunchTicket, String> {
    LAUNCH_CONTROLLER
        .lock()
        .map_err(|error| error.to_string())?
        .claim()
}

fn clear_pending_launch(ticket: &LaunchTicket) {
    if let Ok(mut controller) = LAUNCH_CONTROLLER.lock() {
        controller.clear_pending(ticket);
    }
}

fn attach_launch_session(ticket: &LaunchTicket, id: String) -> bool {
    LAUNCH_CONTROLLER
        .lock()
        .is_ok_and(|mut controller| controller.attach_session(ticket, id))
}

fn finish_launch_session(id: &str) {
    if let Ok(mut controller) = LAUNCH_CONTROLLER.lock() {
        controller.finish_session(id);
    }
}

async fn wait_for_launch<T>(
    cancelled: &mut watch::Receiver<bool>,
    operation: impl Future<Output = T>,
) -> Result<T, String> {
    if *cancelled.borrow() {
        return Err(LAUNCH_CANCELLED.into());
    }
    tokio::select! {
        value = operation => Ok(value),
        changed = cancelled.changed() => match changed {
            Ok(()) if *cancelled.borrow() => Err(LAUNCH_CANCELLED.into()),
            Ok(()) => Err(LAUNCH_CANCELLED.into()),
            Err(_) => Err(LAUNCH_CANCELLED.into()),
        },
    }
}

#[derive(Clone, serde::Serialize)]
pub struct LaunchProgress {
    stage: String,
    progress: f64,
    downloading: bool,
    failure: Option<crate::minecraft::launch::session::LaunchFailure>,
}

impl LaunchProgress {
    fn new(stage: &str, progress: f64, downloading: bool) -> Self {
        Self {
            stage: stage.into(),
            progress,
            downloading,
            failure: None,
        }
    }

    fn failed(failure: crate::minecraft::launch::session::LaunchFailure) -> Self {
        Self {
            stage: "Minecraft.Launch.Stage.WaitWindow".into(),
            progress: 0.92,
            downloading: false,
            failure: Some(failure),
        }
    }
}

#[tauri::command]
pub async fn get_launch_arguments(
    instance_directory: Option<std::path::PathBuf>,
) -> Result<crate::minecraft::launch::arguments::LaunchArguments, String> {
    let state = ConfigManager::instance().app_state.lock().await;
    if let Some(path) = instance_directory {
        current_instance(&state, &path).map_err(|error| error.to_string())?;
        Ok(state
            .instance_launch_arguments
            .get(&path)
            .cloned()
            .unwrap_or_default())
    } else {
        Ok(state
            .launch_arguments
            .clone()
            .unwrap_or_else(crate::minecraft::launch::arguments::LaunchArguments::global_default))
    }
}

#[tauri::command]
pub async fn set_launch_arguments(
    instance_directory: Option<std::path::PathBuf>,
    settings: crate::minecraft::launch::arguments::LaunchArguments,
) -> Result<crate::minecraft::launch::arguments::LaunchArguments, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            crate::minecraft::launch::arguments::update(
                state,
                instance_directory.as_deref(),
                settings,
            )
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_instance_launch_settings(
    instance_directory: std::path::PathBuf,
) -> Result<crate::minecraft::launch::settings::InstanceLaunchSettings, String> {
    let state = ConfigManager::instance().app_state.lock().await;
    current_instance(&state, &instance_directory).map_err(|error| error.to_string())?;
    Ok(state
        .instance_launch_settings
        .get(&instance_directory)
        .cloned()
        .unwrap_or_default())
}

#[tauri::command]
pub async fn set_instance_launch_settings(
    instance_directory: std::path::PathBuf,
    settings: crate::minecraft::launch::settings::InstanceLaunchSettings,
) -> Result<crate::minecraft::launch::settings::InstanceLaunchSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            crate::minecraft::launch::settings::update(state, &instance_directory, settings)
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_proxy_settings() -> crate::io::net::proxy::ProxySettings {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .proxy
        .clone()
}

#[tauri::command]
pub async fn set_proxy_settings(
    settings: crate::io::net::proxy::ProxySettings,
) -> Result<crate::io::net::proxy::ProxySettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update_proxy_settings(settings)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn get_isolation_default() -> crate::minecraft::instance::isolation::IsolationDefault {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .isolation_default
}

#[tauri::command]
pub async fn set_isolation_default(
    value: crate::minecraft::instance::isolation::IsolationDefault,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance()
            .update(|state| crate::minecraft::instance::isolation::set_default(state, value))
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
pub async fn get_memory_settings(
    instance_directory: Option<std::path::PathBuf>,
) -> Result<crate::minecraft::launch::memory::MemorySnapshot, String> {
    use crate::minecraft::launch::memory::{MemoryCustom, MemorySettings, MemorySnapshot};
    let state = ConfigManager::instance().app_state.lock().await;
    match instance_directory {
        Some(directory) => {
            current_instance(&state, &directory).map_err(|error| error.to_string())?;
            let settings = state.instance_memory_settings.get(&directory).cloned();
            let custom = state
                .instance_memory_custom
                .get(&directory)
                .cloned()
                .unwrap_or_else(|| MemoryCustom::from_settings(settings.as_ref()));
            Ok(MemorySnapshot { settings, custom })
        }
        None => {
            let settings = Some(
                state
                    .memory_settings
                    .clone()
                    .unwrap_or(MemorySettings::Custom {
                        maximum_mib: u32::try_from(state.launcher_setup.max_memory)
                            .map_err(|error| error.to_string())?,
                        initial_mib: None,
                    }),
            );
            let custom = state
                .memory_custom
                .clone()
                .unwrap_or_else(|| MemoryCustom::from_settings(settings.as_ref()));
            Ok(MemorySnapshot { settings, custom })
        }
    }
}

#[tauri::command]
pub async fn set_memory_settings(
    instance_directory: Option<std::path::PathBuf>,
    settings: Option<crate::minecraft::launch::memory::MemorySettings>,
    custom: Option<crate::minecraft::launch::memory::MemoryCustom>,
    migrate_only: bool,
) -> Result<crate::minecraft::launch::memory::MemorySnapshot, String> {
    use crate::minecraft::launch::memory::{MemoryCustom, MemorySnapshot};
    if let Some(settings) = &settings {
        settings.validate()?;
    }
    if let Some(custom) = &custom {
        custom.validate()?;
    }
    if let (
        Some(crate::minecraft::launch::memory::MemorySettings::Custom {
            maximum_mib,
            initial_mib,
        }),
        Some(custom),
    ) = (&settings, &custom)
    {
        if *maximum_mib != custom.maximum_mib
            || *initial_mib
                != custom
                    .initial_mib
                    .map(|value| value.min(custom.maximum_mib))
        {
            return Err("Custom memory values do not match the active settings".into());
        }
    }
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            if let Some(directory) = instance_directory {
                current_instance(state, &directory).map_err(|error| error.to_string())?;
                if migrate_only && let Some(saved) = state.instance_memory_settings.get(&directory)
                {
                    return Ok(MemorySnapshot {
                        settings: Some(saved.clone()),
                        custom: state
                            .instance_memory_custom
                            .get(&directory)
                            .cloned()
                            .unwrap_or_else(|| MemoryCustom::from_settings(Some(saved))),
                    });
                }
                if let Some(settings) = settings.clone() {
                    state
                        .instance_memory_settings
                        .insert(directory.clone(), settings);
                } else {
                    state.instance_memory_settings.remove(&directory);
                }
                let custom =
                    custom.unwrap_or_else(|| MemoryCustom::from_settings(settings.as_ref()));
                state
                    .instance_memory_custom
                    .insert(directory.clone(), custom.clone());
                state.edited_memory_instances.insert(directory);
                Ok(MemorySnapshot { settings, custom })
            } else {
                let value = settings
                    .clone()
                    .ok_or("Global memory settings cannot inherit")?;
                if migrate_only && let Some(saved) = &state.memory_settings {
                    let saved = saved.clone();
                    if state.memory_custom.is_none() {
                        state.memory_custom = match &saved {
                            crate::minecraft::launch::memory::MemorySettings::Custom { .. } => {
                                Some(MemoryCustom::from_settings(Some(&saved)))
                            }
                            _ => custom,
                        };
                    }
                    let custom = state
                        .memory_custom
                        .clone()
                        .unwrap_or_else(|| MemoryCustom::from_settings(Some(&saved)));
                    return Ok(MemorySnapshot {
                        settings: Some(saved),
                        custom,
                    });
                }
                state.memory_settings = Some(value.clone());
                let custom = custom.unwrap_or_else(|| MemoryCustom::from_settings(Some(&value)));
                state.memory_custom = Some(custom.clone());
                Ok(MemorySnapshot {
                    settings: Some(value),
                    custom,
                })
            }
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn migrate_instance_memory_settings(
    legacy: std::collections::HashMap<String, crate::minecraft::launch::memory::MemorySnapshot>,
) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().migrate_instance_memory_settings(legacy)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[derive(serde::Serialize)]
pub struct MemoryPreview {
    total_bytes: u64,
    available_bytes: u64,
    allocation_mib: Option<usize>,
}

#[tauri::command]
pub async fn preview_memory_settings(
    instance_directory: Option<std::path::PathBuf>,
    settings: Option<crate::minecraft::launch::memory::MemorySettings>,
    is_64_bit: Option<bool>,
) -> Result<MemoryPreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        use crate::minecraft::launch::memory::{self, MemorySettings};
        let config = ConfigManager::instance();
        let path = {
            let state = config.app_state.blocking_lock();
            instance_directory.clone().or_else(|| {
                state
                    .active_game_instance
                    .as_ref()
                    .map(|instance| instance.directory.clone())
            })
        };
        if let Some(path) = &path {
            config.ensure_instance_isolation(path)?;
        }
        let (instance, effective) = {
            let state = config.app_state.blocking_lock();
            let instance = if let Some(path) = &instance_directory {
                Some(
                    current_instance(&state, path)
                        .map_err(|error| error.to_string())?
                        .clone(),
                )
            } else {
                state.active_game_instance.as_deref().cloned()
            };
            let effective = settings
                .or_else(|| state.memory_settings.clone())
                .unwrap_or(MemorySettings::Custom {
                    maximum_mib: u32::try_from(state.launcher_setup.max_memory)
                        .map_err(|error| error.to_string())?,
                    initial_mib: None,
                });
            let game_directory = instance.as_ref().map(|instance| {
                crate::minecraft::instance::isolation::directory(&state, instance).to_path_buf()
            });
            (
                instance.map(|instance| (instance, game_directory.unwrap())),
                effective,
            )
        };
        effective.validate()?;
        let mut system = sysinfo::System::new().map_err(|_| "Physical memory is unavailable")?;
        system.refresh_memory();
        let total = system.total_memory();
        if total == 0 {
            return Err("Physical memory is unavailable".into());
        }
        let available = system.available_memory().min(total);
        let allocation = match is_64_bit {
            None => None,
            Some(is_64_bit) => {
                let value = if let Some((instance, game_directory)) = &instance {
                    memory::resolve(&effective, available, instance, game_directory, is_64_bit)?.0
                } else {
                    let max = match effective {
                        MemorySettings::Auto => memory::automatic(available, false, false, 0)?,
                        MemorySettings::Custom { maximum_mib, .. } => maximum_mib as usize,
                    };
                    if is_64_bit { max } else { max.min(1024) }
                };
                Some(value)
            }
        };
        Ok(MemoryPreview {
            total_bytes: total,
            available_bytes: available,
            allocation_mib: allocation,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn begin_launch() -> Result<(), String> {
    begin_launch_ticket().map(|_| ())
}

#[tauri::command]
pub async fn cancel_launch() -> Result<(), String> {
    let session = LAUNCH_CONTROLLER
        .lock()
        .map_err(|error| error.to_string())?
        .cancel()?;
    cancel_java_install().await;
    if let Some(id) = session {
        crate::minecraft::launch::session::stop_game_session(id)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn launch_game(
    window: tauri::WebviewWindow,
    instance_directory: std::path::PathBuf,
    progress: tauri::ipc::Channel<LaunchProgress>,
) -> Result<(), String> {
    let ticket = claim_launch_ticket()?;
    let result = launch_game_inner(window, instance_directory, progress, ticket.clone()).await;
    if result.is_err() {
        clear_pending_launch(&ticket);
    }
    result
}

async fn launch_game_inner(
    window: tauri::WebviewWindow,
    instance_directory: std::path::PathBuf,
    progress: tauri::ipc::Channel<LaunchProgress>,
    mut ticket: LaunchTicket,
) -> Result<(), String> {
    let report = |stage, value, downloading| {
        let _ = progress.send(LaunchProgress::new(stage, value, downloading));
    };
    report("Minecraft.Launch.Stage.GetJava", 0.08, false);
    let info = wait_for_launch(
        &mut ticket.receiver,
        java::preference::prepare(instance_directory.clone()),
    )
    .await??;
    let runtime = info.runtime.ok_or_else(|| {
        format!(
            "No compatible Java found: {}",
            info.requirements.description()
        )
    })?;
    let expected = instance_directory.clone();
    report("Minecraft.Launch.Stage.GetArguments", 0.2, false);
    let mut option = wait_for_launch(
        &mut ticket.receiver,
        tauri::async_runtime::spawn_blocking(move || {
            ConfigManager::instance().ensure_instance_isolation(&instance_directory)?;
            let state = ConfigManager::instance().app_state.blocking_lock();
            current_instance(&state, &instance_directory).map_err(|e| e.to_string())?;
            let option =
                LaunchOption::from_state(&state, Arc::new(runtime)).map_err(|e| e.to_string())?;
            Ok::<_, String>(option)
        }),
    )
    .await?
    .map_err(|error| error.to_string())??;
    if option.uses_launcher_size() {
        if ticket.cancelled.load(Ordering::Acquire) {
            return Err(LAUNCH_CANCELLED.into());
        }
        let size = window.inner_size().map_err(|error| error.to_string())?;
        if size.width == 0 || size.height == 0 {
            return Err("Launcher window size is unavailable".into());
        }
        option = option.with_window_size(size.width as usize, size.height as usize);
    }
    let config = ConfigManager::instance();
    let downloader = crate::io::download::Downloader::with_bases(
        wait_for_launch(&mut ticket.receiver, config.api_client.api_bases_async()).await?,
    );
    report("Minecraft.Launch.Stage.CompleteFiles", 0.35, true);
    let natives = wait_for_launch(&mut ticket.receiver, option.prepare_natives(&downloader))
        .await?
        .map_err(|e| format!("{e:#}"))?;
    report("Minecraft.Launch.Stage.CompleteFiles", 0.65, true);
    wait_for_launch(
        &mut ticket.receiver,
        option.prepare_legacy_assets(&downloader),
    )
    .await?
    .map_err(|e| format!("{e:#}"))?;
    report("Minecraft.Launch.Stage.StartProcess", 0.85, false);
    let cancelled = ticket.cancelled.clone();
    let access_token = option.access_token();
    let launch_instance = expected.clone();
    let session_ticket = ticket.clone();
    let session_progress = progress.clone();
    wait_for_launch(
        &mut ticket.receiver,
        tauri::async_runtime::spawn_blocking(move || {
            if cancelled.load(Ordering::Acquire) {
                return Err(LAUNCH_CANCELLED.to_string());
            }
            let state = config.app_state.blocking_lock();
            current_instance(&state, &launch_instance).map_err(|e| e.to_string())?;
            let visibility = state.launcher_visibility;
            let mut child = option.launch_checked().map_err(|e| e.to_string())?;
            if cancelled.load(Ordering::Acquire) {
                let _ = child.kill();
                let _ = child.wait();
                return Err(LAUNCH_CANCELLED.to_string());
            }
            let id = crate::minecraft::launch::session::track(
                child,
                expected,
                access_token,
                move || drop(natives),
                move |id, event| {
                    match event {
                        crate::minecraft::launch::session::SessionEvent::Spawned => {
                            let _ = session_progress.send(LaunchProgress::new(
                                "Minecraft.Launch.Stage.WaitWindow",
                                0.92,
                                false,
                            ));
                        }
                        crate::minecraft::launch::session::SessionEvent::Ready => {
                            let _ = session_progress.send(LaunchProgress::new(
                                "Minecraft.Launch.Stage.End",
                                1.0,
                                false,
                            ));
                            finish_launch_session(id);
                        }
                        crate::minecraft::launch::session::SessionEvent::Finished {
                            ref failure,
                            ..
                        } => {
                            if let Some(failure) = failure {
                                let _ =
                                    session_progress.send(LaunchProgress::failed(failure.clone()));
                            }
                            finish_launch_session(id);
                        }
                    }
                    crate::app::visibility::handle(&window, id, visibility, event)
                },
            );
            if !attach_launch_session(&session_ticket, id.clone()) {
                let _ = crate::minecraft::launch::session::stop_game_session(id);
                return Err(LAUNCH_CANCELLED.to_string());
            }
            Ok::<_, String>(())
        }),
    )
    .await?
    .map_err(|error| error.to_string())??;
    Ok(())
}

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
    java::refresh(true).await
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
        let global_is_64_bit = java::preference::is_game_set_64_bit(
            &runtimes,
            global_java.as_deref(),
            &JavaPreference::Global,
        );
        let instance_is_64_bit = java::preference::is_game_set_64_bit(
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
        java::preference::validate_preference(&validated.preference)
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(|e| e.to_string())?;
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            current_instance(&state, &instance_directory).map_err(|e| e.to_string())?;
            state.java_preferences.insert(instance_directory, settings);
            Ok(())
        })
    })
    .await
    .map_err(|e| e.to_string())?
}

#[tauri::command]
pub async fn resolve_legacy_instance_ids(
    ids: std::collections::HashSet<String>,
) -> Result<std::collections::HashMap<String, std::path::PathBuf>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().resolve_legacy_instance_ids(ids)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn migrate_instance_java_settings(
    legacy: std::collections::HashMap<String, InstanceJavaSettings>,
) -> Result<Vec<String>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().migrate_instance_java_settings(legacy)
    })
    .await
    .map_err(|error| error.to_string())?
}
#[tauri::command]
pub async fn select_relative_java(app: AppHandle) -> Result<Option<JavaPreference>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let root = java::launcher_directory().map_err(|e| e.to_string())?;
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
        java::preference::relative_java(&root, &relative).map_err(|e| e.to_string())?;
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
pub async fn open_instance_folder(
    instance_directory: std::path::PathBuf,
    kind: McInstanceFolder,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().ensure_instance_isolation(&instance_directory)?;
        let state = ConfigManager::instance().app_state.blocking_lock();
        let instance =
            current_instance(&state, &instance_directory).map_err(|error| error.to_string())?;
        let game_directory = crate::minecraft::instance::isolation::directory(&state, instance);
        let directory =
            crate::minecraft::instance::prepare_shortcut_directory(instance, game_directory, kind)
                .map_err(|error| error.to_string())?;
        open_directory(&directory)
    })
    .await
    .map_err(|error| error.to_string())?
}
#[tauri::command]
pub async fn prepare_game_java(
    instance_directory: std::path::PathBuf,
) -> Result<JavaLaunchInfo, String> {
    java::preference::prepare(instance_directory).await
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
    let info = java::preference::prepare(instance_directory).await?;
    let config = ConfigManager::instance();
    let bases = config.api_client.api_bases_async().await;
    let task = tauri::async_runtime::spawn(java::install::install(
        info.requirements,
        config.java_runtime_dir(),
        bases,
        progress,
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
async fn cancel_java_install() {
    if let Some(task) = JAVA_INSTALL.lock().await.as_ref() {
        task.abort();
    }
}

#[tauri::command]
pub async fn cancel_java_download() {
    cancel_java_install().await;
}

#[tauri::command]
pub fn get_repositories(state: State<'_, Arc<Mutex<AppState>>>) -> Vec<McFolder> {
    let state = state.blocking_lock();
    state.repositories.clone()
}

#[tauri::command(rename_all = "snake_case")]
pub async fn select_repository(repository_index: usize) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().select_repository(repository_index)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command(rename_all = "snake_case")]
pub fn add_new_repository(
    new_repo_path: &str,
    new_repo_name: &str,
) -> Result<Vec<McFolder>, String> {
    ConfigManager::instance().add_repository(new_repo_name, std::path::Path::new(new_repo_path))
}

/// async commands that contain references as inputs must return a Result
/// so this place returns an option
#[tauri::command]
pub async fn get_account() -> Option<SafeProfile> {
    let account = &ConfigManager::instance()
        .app_state
        .lock()
        .await
        .active_account;
    account.as_deref().map(SafeProfile::from)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_instances_in_repository(
    state: State<'_, Arc<Mutex<AppState>>>,
    repository_index: usize,
) -> Result<Vec<McInstance>, String> {
    let repo = state
        .lock()
        .await
        .repositories
        .get(repository_index)
        .cloned()
        .ok_or("Minecraft folder index is out of range")?;
    tauri::async_runtime::spawn_blocking(move || repo.game_instances())
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command(rename_all = "snake_case")]
pub async fn instance_folder_action(
    repository_index: usize,
    instance_name: String,
    instance_directory: std::path::PathBuf,
    delete: bool,
) -> Result<(), String> {
    crate::io::files::validate_file_name(&instance_name).map_err(|e| e.to_string())?;
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

/// Resolve the instance from the configured repositories and build its launch plan.
async fn prepare_launch_option(
    instance_directory: std::path::PathBuf,
) -> Result<LaunchOption, String> {
    let info = java::preference::prepare(instance_directory.clone()).await?;
    let runtime = info.runtime.ok_or_else(|| {
        format!(
            "No compatible Java found: {}",
            info.requirements.description()
        )
    })?;
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().ensure_instance_isolation(&instance_directory)?;
        let state = ConfigManager::instance().app_state.blocking_lock();
        current_instance(&state, &instance_directory).map_err(|e| e.to_string())?;
        LaunchOption::from_state(&state, Arc::new(runtime)).map_err(|e| e.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn export_launch_script(
    window: tauri::WebviewWindow,
    instance_directory: std::path::PathBuf,
    target_path: std::path::PathBuf,
) -> Result<(), String> {
    if !target_path.is_absolute() || target_path.file_name().is_none() {
        return Err("Invalid launch script path".into());
    }
    let mut option = prepare_launch_option(instance_directory).await?;
    if option.uses_launcher_size() {
        let size = window.inner_size().map_err(|error| error.to_string())?;
        if size.width == 0 || size.height == 0 {
            return Err("Launcher window size is unavailable".into());
        }
        option = option.with_window_size(size.width as usize, size.height as usize);
    }
    let downloader = crate::io::download::Downloader::with_bases(
        ConfigManager::instance().api_client.api_bases_async().await,
    );
    let natives = option
        .prepare_natives(&downloader)
        .await
        .map_err(|e| format!("{e:#}"))?;
    option
        .prepare_legacy_assets(&downloader)
        .await
        .map_err(|e| format!("{e:#}"))?;
    // The exported script keeps pointing at this natives folder, so it must survive.
    std::mem::forget(natives);
    option
        .export_script(&target_path)
        .map_err(|error| error.to_string())?;
    Ok(())
}

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
    crate::io::files::validate_file_name(&new_name).map_err(|error| error.to_string())?;
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
    crate::minecraft::instance::rename_instance_files(
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
        if state.edited_memory_instances.remove(&instance.directory) {
            state.edited_memory_instances.insert(new_directory.clone());
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
    crate::io::files::validate_file_name(&instance_name).map_err(|error| error.to_string())?;
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

#[tauri::command]
pub async fn get_version_manifest(
    client: State<'_, &api_client::MinecraftApiClient>,
) -> Result<api_client::game::McVersionManifest, String> {
    match client
        .get_with_endpoint::<api_client::game::McVersionManifest>(
            api_client::game::VERSION_MANIFEST_ENDPOINT,
            true,
        )
        .await
    {
        Ok(manifest) => Ok(manifest),
        Err(e) => return Err(format!("get_version_manifest: {:?}", e)),
    }
}

/// return the version json
#[tauri::command]
pub async fn handle_clicked_on_version(
    client: State<'_, &api_client::MinecraftApiClient>,
    id: &str,
) -> Result<api_client::game::McVersion, String> {
    let temp_dir = std::env::temp_dir().join(format!("pcl-proto-{}", id));
    log::debug!("selected tmp: {:?}", &temp_dir);
    let version_detail = client
        .get_version_details(id, &temp_dir)
        .await
        .map_err(|err| err.to_string())?;
    Ok(version_detail)
}

#[tauri::command(rename_all = "snake_case")]
pub async fn get_plugin_versions(
    api_client: State<'_, &api_client::MinecraftApiClient>,
    plugin_type: mod_loader::ModLoaderType,
    mc_version: &str,
) -> Result<Vec<McPluginReport>, String> {
    let verisons = match plugin_type {
        mod_loader::ModLoaderType::Forge => api_client.get_forge_versions(mc_version).await,
        mod_loader::ModLoaderType::Fabric => {
            api_client
                .get_fabric_versions(mc_version, api_client::FabricFamily::Fabric)
                .await
        }
        mod_loader::ModLoaderType::LegacyFabric => {
            api_client
                .get_fabric_versions(mc_version, api_client::FabricFamily::Legacyfabric)
                .await
        }
        _ => Err(api_client::McApiError::PluginMismatch(plugin_type)),
    };
    verisons.map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::{LaunchController, LaunchProgress};

    #[test]
    fn launch_progress_uses_the_frontend_channel_shape() {
        let value = serde_json::to_value(LaunchProgress::new(
            "Minecraft.Launch.Stage.GetJava",
            0.08,
            false,
        ))
        .unwrap();
        assert_eq!(value["stage"], "Minecraft.Launch.Stage.GetJava");
        assert_eq!(value["progress"], 0.08);
        assert_eq!(value["downloading"], false);
        assert!(value["failure"].is_null());
    }

    #[test]
    fn cancelling_a_pending_launch_releases_the_controller() {
        let mut controller = LaunchController::default();
        let ticket = controller.begin().unwrap();
        assert!(controller.begin().is_err());

        assert_eq!(controller.cancel().unwrap(), None);
        assert!(ticket.cancelled.load(std::sync::atomic::Ordering::Acquire));
        assert!(controller.active.is_none());
        assert!(!controller.attach_session(&ticket, "late-session".into()));
    }

    #[test]
    fn cancelling_a_started_launch_waits_for_session_shutdown() {
        let mut controller = LaunchController::default();
        let ticket = controller.begin().unwrap();
        controller.claim().unwrap();
        assert!(controller.attach_session(&ticket, "session".into()));

        assert_eq!(controller.cancel().unwrap(), Some("session".into()));
        assert!(controller.active.is_some());
        controller.finish_session("session");
        assert!(controller.active.is_none());
    }

    #[tokio::test]
    async fn cancelled_ticket_does_not_start_future_work() {
        let mut controller = LaunchController::default();
        let mut ticket = controller.begin().unwrap();
        controller.cancel().unwrap();

        let result = super::wait_for_launch(&mut ticket.receiver, async { "started" }).await;
        assert_eq!(result.unwrap_err(), super::LAUNCH_CANCELLED);
    }
}
