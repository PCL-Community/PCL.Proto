//! 联机运行时（Npcap）探测与首次按需安装。
//!
//! EasyTier 在 Windows 上优先使用 WinDivert，普通用户没有管理员权限时会回退到
//! PnetTun，而 PnetTun 需要 Npcap 的 `Packet.dll`。这里只在用户真正使用联机
//! 功能时安装 Npcap，安装包仍由用户通过官方安装程序确认。

use serde::Serialize;

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LinkRuntimeState {
    Ready,
    NeedsInstall,
    NeedsRestart,
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
pub struct LinkRuntimeStatus {
    pub state: LinkRuntimeState,
    pub message: Option<String>,
}

#[derive(Serialize, Clone, Debug)]
pub struct LinkRuntimeEvent {
    pub phase: String,
    pub progress: Option<f64>,
    pub speed: Option<f64>,
    pub message: Option<String>,
}

impl LinkRuntimeEvent {
    fn phase(phase: &str) -> Self {
        Self {
            phase: phase.into(),
            progress: None,
            speed: None,
            message: None,
        }
    }

    #[cfg(windows)]
    fn progress(phase: &str, progress: f64, speed: Option<f64>) -> Self {
        Self {
            phase: phase.into(),
            progress: Some(if progress.is_finite() {
                progress.clamp(0.0, 1.0)
            } else {
                0.0
            }),
            speed,
            message: None,
        }
    }

    #[cfg(windows)]
    fn message(phase: &str, message: impl Into<String>) -> Self {
        Self {
            phase: phase.into(),
            progress: None,
            speed: None,
            message: Some(message.into()),
        }
    }
}

fn ready() -> LinkRuntimeStatus {
    LinkRuntimeStatus {
        state: LinkRuntimeState::Ready,
        message: None,
    }
}

#[cfg(not(windows))]
#[tauri::command]
pub fn get_link_runtime_status() -> LinkRuntimeStatus {
    ready()
}

#[cfg(not(windows))]
#[tauri::command(rename_all = "snake_case")]
pub async fn install_link_runtime(
    on_event: tauri::ipc::Channel<LinkRuntimeEvent>,
) -> Result<LinkRuntimeStatus, String> {
    let _ = on_event.send(LinkRuntimeEvent::phase("ready"));
    Ok(ready())
}

#[cfg(windows)]
mod imp {
    use super::{LinkRuntimeEvent, LinkRuntimeState, LinkRuntimeStatus};
    use pcl_core::io::download::TaskItemReport;
    use std::{
        ffi::OsStr,
        path::{Path, PathBuf},
        sync::atomic::{AtomicI32, Ordering},
    };

    const NPCAP_VERSION: &str = "1.90";
    const NPCAP_URL: &str = "https://npcap.com/dist/npcap-1.90.exe";
    // Kept in sync with the previous install script; the installer is executed elevated.
    const NPCAP_SHA256: &str = "682B57E1C05EF375931975C9D6DED40888F4BFDF9F4B11571E654F962F1199B6";

    static NEXT_DOWNLOAD_TASK: AtomicI32 = AtomicI32::new(-100);
    static INSTALL_LOCK: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

    fn system_root() -> PathBuf {
        std::env::var_os("SystemRoot")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(r"C:\Windows"))
    }

    fn service_installed(name: &str) -> bool {
        use winreg::{RegKey, enums::HKEY_LOCAL_MACHINE};
        RegKey::predef(HKEY_LOCAL_MACHINE)
            .open_subkey(format!(r"SYSTEM\CurrentControlSet\Services\{name}"))
            .is_ok()
    }

    fn driver_installed() -> bool {
        let drivers = system_root().join("System32").join("drivers");
        drivers.join("npcap.sys").is_file() || drivers.join("npf.sys").is_file()
    }

    fn packet_dll_loadable() -> bool {
        pnet_datalink::winpcap::is_available()
    }

    pub(super) fn probe() -> LinkRuntimeStatus {
        let service = service_installed("npcap") || service_installed("npf");
        let driver = driver_installed();
        let api = packet_dll_loadable();
        if service && driver && api {
            super::ready()
        } else if service && driver {
            LinkRuntimeStatus {
                state: LinkRuntimeState::NeedsRestart,
                message: Some("Npcap is installed but Packet.dll could not be loaded".into()),
            }
        } else {
            LinkRuntimeStatus {
                state: LinkRuntimeState::NeedsInstall,
                message: None,
            }
        }
    }

    fn installer_path() -> PathBuf {
        crate::config::manager::ConfigManager::instance()
            .config_dir()
            .join("runtime")
            .join("link")
            .join(format!("npcap-{NPCAP_VERSION}.exe"))
    }

    fn wide_null(value: impl AsRef<OsStr>) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        value
            .as_ref()
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    fn sha256_file(path: &Path) -> Result<String, String> {
        use sha2::{Digest, Sha256};
        use std::io::Read;
        let mut file = std::fs::File::open(path).map_err(|error| error.to_string())?;
        let mut hasher = Sha256::new();
        let mut buffer = [0_u8; 64 * 1024];
        loop {
            let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
            if read == 0 {
                break;
            }
            hasher.update(&buffer[..read]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }

    fn verify_authenticode(path: &Path) -> bool {
        use windows_sys::Win32::Security::WinTrust::{
            WINTRUST_ACTION_GENERIC_VERIFY_V2, WINTRUST_DATA, WINTRUST_FILE_INFO, WTD_CHOICE_FILE,
            WTD_REVOKE_NONE, WTD_STATEACTION_CLOSE, WTD_STATEACTION_VERIFY, WTD_UI_NONE,
            WinVerifyTrust,
        };
        let file = wide_null(path.as_os_str());
        let mut file_info = WINTRUST_FILE_INFO {
            cbStruct: std::mem::size_of::<WINTRUST_FILE_INFO>() as u32,
            pcwszFilePath: file.as_ptr(),
            hFile: std::ptr::null_mut(),
            pgKnownSubject: std::ptr::null_mut(),
        };
        let mut data: WINTRUST_DATA = unsafe { std::mem::zeroed() };
        data.cbStruct = std::mem::size_of::<WINTRUST_DATA>() as u32;
        data.dwUIChoice = WTD_UI_NONE;
        data.fdwRevocationChecks = WTD_REVOKE_NONE;
        data.dwUnionChoice = WTD_CHOICE_FILE;
        data.Anonymous.pFile = &mut file_info;
        data.dwStateAction = WTD_STATEACTION_VERIFY;
        let mut action = WINTRUST_ACTION_GENERIC_VERIFY_V2;
        let status = unsafe {
            WinVerifyTrust(
                std::ptr::null_mut(),
                &mut action,
                &mut data as *mut _ as *mut _,
            )
        };
        data.dwStateAction = WTD_STATEACTION_CLOSE;
        unsafe {
            WinVerifyTrust(
                std::ptr::null_mut(),
                &mut action,
                &mut data as *mut _ as *mut _,
            );
        }
        status == 0
    }

    fn verify_installer(path: &Path) -> Result<(), String> {
        let size = std::fs::metadata(path)
            .map_err(|error| error.to_string())?
            .len();
        if size < 1024 * 1024 {
            return Err("Npcap installer is unexpectedly small".into());
        }
        let hash = sha256_file(path)?;
        if !hash.eq_ignore_ascii_case(NPCAP_SHA256) {
            return Err("Npcap installer checksum mismatch".into());
        }
        if !verify_authenticode(path) {
            return Err("Npcap installer signature is not trusted".into());
        }
        Ok(())
    }

    async fn download_installer(
        on_event: &tauri::ipc::Channel<LinkRuntimeEvent>,
    ) -> Result<PathBuf, String> {
        let path = installer_path();
        if path.is_file() {
            let _ = on_event.send(LinkRuntimeEvent::phase("verifying"));
            let cached = path.clone();
            if tokio::task::spawn_blocking(move || verify_installer(&cached))
                .await
                .map_err(|error| error.to_string())?
                .is_ok()
            {
                let _ = on_event.send(LinkRuntimeEvent::progress("downloading", 1.0, None));
                return Ok(path);
            }
            let _ = std::fs::remove_file(&path);
        }

        let parent = path
            .parent()
            .ok_or_else(|| "Invalid Npcap installer path".to_string())?;
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let temporary = parent.join(format!(
            "npcap-{}-{}.download",
            NPCAP_VERSION,
            uuid::Uuid::new_v4()
        ));

        let downloader = crate::config::manager::ConfigManager::instance()
            .api_client
            .downloader()
            .await;
        let task_id = NEXT_DOWNLOAD_TASK.fetch_sub(1, Ordering::Relaxed);
        let event = on_event.clone();
        let sink = pcl_core::events::event_sink(move |report: TaskItemReport| {
            let _ = event.send(LinkRuntimeEvent::progress(
                "downloading",
                report.progress,
                report.speed,
            ));
        });
        let result = pcl_core::io::download::download_custom_file(
            &downloader,
            NPCAP_URL.to_string(),
            temporary.clone(),
            task_id,
            sink,
        )
        .await;
        if let Err(error) = result {
            let _ = std::fs::remove_file(&temporary);
            return Err(format!("Could not download the Npcap installer: {error}"));
        }

        let _ = on_event.send(LinkRuntimeEvent::phase("verifying"));
        let check = temporary.clone();
        let verified = tokio::task::spawn_blocking(move || verify_installer(&check))
            .await
            .map_err(|error| error.to_string())?;
        if let Err(error) = verified {
            let _ = std::fs::remove_file(&temporary);
            return Err(error);
        }
        if path.exists() {
            std::fs::remove_file(&path).map_err(|error| error.to_string())?;
        }
        std::fs::rename(&temporary, &path).map_err(|error| error.to_string())?;
        let _ = on_event.send(LinkRuntimeEvent::progress("downloading", 1.0, None));
        Ok(path)
    }

    enum InstallOutcome {
        Installed(u32),
        Cancelled,
    }

    fn run_installer(path: &Path) -> Result<InstallOutcome, String> {
        use windows_sys::Win32::Foundation::{
            CloseHandle, ERROR_CANCELLED, GetLastError, WAIT_OBJECT_0,
        };
        use windows_sys::Win32::System::Threading::{
            GetExitCodeProcess, INFINITE, WaitForSingleObject,
        };
        use windows_sys::Win32::UI::Shell::{
            SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW,
        };
        use windows_sys::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;

        let file = wide_null(path.as_os_str());
        let verb = wide_null("runas");
        let parameters = wide_null("/winpcap_mode=yes /admin_only=no");
        let mut info: SHELLEXECUTEINFOW = unsafe { std::mem::zeroed() };
        info.cbSize = std::mem::size_of::<SHELLEXECUTEINFOW>() as u32;
        info.fMask = SEE_MASK_NOCLOSEPROCESS;
        info.lpVerb = verb.as_ptr();
        info.lpFile = file.as_ptr();
        info.lpParameters = parameters.as_ptr();
        info.nShow = SW_SHOWNORMAL;
        if unsafe { ShellExecuteExW(&mut info) } == 0 {
            let error = unsafe { GetLastError() };
            if error == ERROR_CANCELLED {
                return Ok(InstallOutcome::Cancelled);
            }
            return Err(format!(
                "Could not start the Npcap installer (Windows error {error})"
            ));
        }
        if info.hProcess.is_null() {
            return Err("Npcap installer did not start".into());
        }
        let wait = unsafe { WaitForSingleObject(info.hProcess, INFINITE) };
        let mut code = 0_u32;
        let read_code = unsafe { GetExitCodeProcess(info.hProcess, &mut code) };
        unsafe { CloseHandle(info.hProcess) };
        if wait != WAIT_OBJECT_0 || read_code == 0 {
            return Err("Could not wait for the Npcap installer".into());
        }
        Ok(InstallOutcome::Installed(code))
    }

    pub(super) fn status() -> LinkRuntimeStatus {
        probe()
    }

    pub(super) async fn install(
        on_event: tauri::ipc::Channel<LinkRuntimeEvent>,
    ) -> Result<LinkRuntimeStatus, String> {
        let _guard = INSTALL_LOCK.lock().await;
        let _ = on_event.send(LinkRuntimeEvent::phase("checking"));
        let status = probe();
        match status.state {
            LinkRuntimeState::Ready => {
                let _ = on_event.send(LinkRuntimeEvent::phase("ready"));
                return Ok(status);
            }
            LinkRuntimeState::NeedsRestart => {
                let _ = on_event.send(LinkRuntimeEvent::phase("needs_restart"));
                return Ok(status);
            }
            LinkRuntimeState::NeedsInstall => {}
        }

        let installer = download_installer(&on_event).await?;

        let _ = on_event.send(LinkRuntimeEvent::phase("installing"));
        let outcome = tokio::task::spawn_blocking(move || run_installer(&installer))
            .await
            .map_err(|error| error.to_string())??;
        match outcome {
            InstallOutcome::Cancelled => {
                let _ = on_event.send(LinkRuntimeEvent::phase("cancelled"));
                Ok(status)
            }
            InstallOutcome::Installed(0) => {
                let status = probe();
                if status.state == LinkRuntimeState::NeedsInstall {
                    let message = "Npcap installation did not become available";
                    let _ = on_event.send(LinkRuntimeEvent::message("error", message));
                    return Ok(LinkRuntimeStatus {
                        state: LinkRuntimeState::NeedsInstall,
                        message: Some(message.into()),
                    });
                }
                let _ = on_event.send(LinkRuntimeEvent::phase(match status.state {
                    LinkRuntimeState::Ready => "ready",
                    LinkRuntimeState::NeedsRestart => "needs_restart",
                    LinkRuntimeState::NeedsInstall => "error",
                }));
                Ok(status)
            }
            InstallOutcome::Installed(3010 | 1641) => {
                let _ = on_event.send(LinkRuntimeEvent::phase("needs_restart"));
                Ok(LinkRuntimeStatus {
                    state: LinkRuntimeState::NeedsRestart,
                    message: None,
                })
            }
            InstallOutcome::Installed(code) => {
                let message = format!("Npcap installer exited with code {code}");
                let _ = on_event.send(LinkRuntimeEvent::message("error", message.clone()));
                Err(message)
            }
        }
    }
}

#[cfg(windows)]
#[tauri::command]
pub fn get_link_runtime_status() -> LinkRuntimeStatus {
    imp::status()
}

#[cfg(windows)]
#[tauri::command(rename_all = "snake_case")]
pub async fn install_link_runtime(
    on_event: tauri::ipc::Channel<LinkRuntimeEvent>,
) -> Result<LinkRuntimeStatus, String> {
    imp::install(on_event).await
}
