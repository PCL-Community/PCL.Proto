//! 桌面应用的目录约定：启动器旁边的 `.minecraft`、官方启动器目录等。

use anyhow::Context;
use std::path::{Path, PathBuf};

use pcl_core::utils::platform::OS;

/// 可执行文件旁边的 `.minecraft`（AppImage 使用原始路径）。
pub fn launcher_minecraft_directory() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?.canonicalize().ok()?;
    let appimage = std::env::var_os("APPIMAGE").map(PathBuf::from);
    let os = OS::current();
    if os == OS::macOS {
        return None;
    }
    // An AppImage's executable lives in a temporary, read-only mount.
    let executable = if os == OS::Linux {
        appimage
            .as_deref()
            .filter(|path| path.is_absolute() && path.is_file())
            .unwrap_or(&executable)
    } else {
        &executable
    };
    Some(executable.parent()?.join(".minecraft"))
}

/// 官方启动器的游戏目录。
pub fn official_minecraft_directory() -> Option<PathBuf> {
    let dirs = directories::BaseDirs::new()?;
    Some(match OS::current() {
        OS::Windows => dirs.config_dir().join(".minecraft"),
        OS::Linux => dirs.home_dir().join(".minecraft"),
        OS::macOS => dirs
            .home_dir()
            .join("Library/Application Support/minecraft"),
    })
}

/// 首次启动时准备一个可写的游戏目录：优先便携目录，否则落在配置目录下。
pub fn create_default_directory(
    config_dir: &Path,
    portable: Option<&Path>,
) -> std::io::Result<PathBuf> {
    let prepare = |path: &Path| -> std::io::Result<PathBuf> {
        std::fs::create_dir_all(path.join("versions"))?;
        tempfile::tempfile_in(path)?;
        path.canonicalize()
    };
    if let Some(path) = portable {
        match prepare(path) {
            Ok(path) => return Ok(path),
            Err(error) => log::warn!(
                "Cannot use launcher-local Minecraft directory {}: {error}; using config directory",
                path.display()
            ),
        }
    }
    prepare(&config_dir.join(".minecraft"))
}

pub fn launcher_directory() -> anyhow::Result<PathBuf> {
    Ok(std::env::current_exe()?
        .parent()
        .context("Missing launcher directory")?
        .to_owned())
}
