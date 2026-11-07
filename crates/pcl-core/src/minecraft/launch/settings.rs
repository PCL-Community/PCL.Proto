use std::{path::Path, process::Command};

/// Concrete launch behaviors supplied by the caller.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LaunchExtras {
    pub info: String,
    /// 0 = game default, 1 = llvmpipe, 2 = d3d12, 3 = zink.
    pub renderer: u8,
    pub prelaunch_command: String,
    pub prelaunch_wait: bool,
    pub disable_asset_verify: bool,
    pub java_wrapper: bool,
    pub legacy_fix: bool,
    pub debug_log4j2: bool,
    pub lwjgl_unsafe_agent: bool,
    pub proxy: Option<(String, u16)>,
}

/// CE only uses the Java wrapper for legacy versions under a UTF-8 Windows code page.
pub fn java_wrapper_supported() -> bool {
    if crate::utils::platform::host_architecture() == "aarch64" {
        return false;
    }
    #[cfg(windows)]
    {
        // SAFETY: GetACP has no preconditions.
        unsafe { windows_sys::Win32::Globalization::GetACP() == 65001 }
    }
    #[cfg(not(windows))]
    {
        false
    }
}

/// Run the pre-launch command like CE: failures are logged, never fatal.
pub fn run_prelaunch(extras: &LaunchExtras, working_directory: &Path) {
    let command = extras.prelaunch_command.trim();
    if command.is_empty() {
        return;
    }
    let mut process = if cfg!(windows) {
        let mut process = Command::new("cmd");
        process.arg("/c").arg(command);
        process
    } else {
        let mut process = Command::new("sh");
        process.arg("-c").arg(command);
        process
    };
    process
        .current_dir(working_directory)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null());
    match process.spawn() {
        Ok(mut child) => {
            if extras.prelaunch_wait
                && let Err(error) = child.wait()
            {
                log::warn!("Pre-launch command did not finish: {error}");
            }
        }
        Err(error) => log::warn!("Cannot run pre-launch command: {error}"),
    }
}
