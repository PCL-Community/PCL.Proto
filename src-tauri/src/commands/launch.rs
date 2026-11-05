use super::java::cancel_java_download;
use crate::{
    app::config::ConfigManager,
    minecraft::{
        java::{self, JavaEntry, preference::current_instance},
        launch::launcher::LaunchOption,
    },
};
use std::{
    future::Future,
    sync::{
        Arc, LazyLock, Mutex as StdMutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::watch;

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
pub fn begin_launch() -> Result<(), String> {
    begin_launch_ticket().map(|_| ())
}

#[tauri::command]
pub async fn cancel_launch() -> Result<(), String> {
    let session = LAUNCH_CONTROLLER
        .lock()
        .map_err(|error| error.to_string())?
        .cancel()?;
    cancel_java_download().await;
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
    server_address: Option<String>,
) -> Result<(), String> {
    let ticket = claim_launch_ticket()?;
    let result = launch_game_inner(
        window,
        instance_directory,
        progress,
        server_address,
        ticket.clone(),
    )
    .await;
    if result.is_err() {
        clear_pending_launch(&ticket);
    }
    result
}

async fn launch_game_inner(
    window: tauri::WebviewWindow,
    instance_directory: std::path::PathBuf,
    progress: tauri::ipc::Channel<LaunchProgress>,
    server_address: Option<String>,
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
            build_launch_option(instance_directory, runtime)
        }),
    )
    .await?
    .map_err(|error| error.to_string())??;
    if let Some(address) = server_address {
        option = option
            .with_server(&address)
            .map_err(|error| error.to_string())?;
    }
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
    wait_for_launch(&mut ticket.receiver, option.prepare_natives(&downloader))
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
                || {},
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
    // CE 在启动前先跑一遍登录加载器：令牌过期时静默刷新，失败就要求重新登录。
    refresh_active_account().await?;
    let option = tauri::async_runtime::spawn_blocking(move || {
        build_launch_option(instance_directory, runtime)
    })
    .await
    .map_err(|error| error.to_string())??;
    prepare_authlib(option).await
}

/// 刷新当前档案的访问令牌，对照 CE 的 `ModLaunch.mcLoginLoader`。
async fn refresh_active_account() -> Result<(), String> {
    use crate::minecraft::profile::{ProfileType, service};
    let Some(profile) = tauri::async_runtime::spawn_blocking(service::current)
        .await
        .map_err(|error| error.to_string())?
    else {
        // 没有档案时由 `LaunchOption::from_state` 报 MissingAccount。
        return Ok(());
    };
    if profile.profile_type == ProfileType::Offline {
        return Ok(());
    }
    crate::minecraft::profile::auth::session::login_for_launch(&profile, false)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

/// 第三方账号需要 authlib-injector 才能连到自己的认证服务器。
async fn prepare_authlib(option: LaunchOption) -> Result<LaunchOption, String> {
    use crate::minecraft::profile::ProfileType;
    let profile = option.account().clone();
    if !matches!(
        profile.profile_type,
        ProfileType::Authlib | ProfileType::YggdrasilConnect
    ) {
        return Ok(option);
    }
    let agent = crate::minecraft::launch::authlib::prepare(&profile).await?;
    Ok(option.with_authlib(agent))
}

fn build_launch_option(
    instance_directory: std::path::PathBuf,
    runtime: JavaEntry,
) -> Result<LaunchOption, String> {
    let config = ConfigManager::instance();
    config.ensure_instance_isolation(&instance_directory)?;
    let state = config.app_state.blocking_lock();
    current_instance(&state, &instance_directory).map_err(|error| error.to_string())?;
    LaunchOption::from_state(&state, Arc::new(runtime)).map_err(|error| error.to_string())
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
    option
        .prepare_natives(&downloader)
        .await
        .map_err(|e| format!("{e:#}"))?;
    option
        .prepare_legacy_assets(&downloader)
        .await
        .map_err(|e| format!("{e:#}"))?;
    option
        .export_script(&target_path)
        .map_err(|error| error.to_string())?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::LaunchController;

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
