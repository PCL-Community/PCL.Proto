//! 启动兼容资源；默认按功能需要准备，也可由宿主打包后传入替代路径。

use crate::io::download::{Downloader, resources::HttpMode};
use std::path::{Path, PathBuf};

pub const JAVA_WRAPPER_NAME: &str = "java-wrapper.jar";
pub const LEGACY_FIX_NAME: &str = "legacyfix.jar";
pub const LWJGL_UNSAFE_AGENT_NAME: &str = "lwjgl-unsafe-agent.jar";
pub const LOG4J2_DEBUG_NAME: &str = "log4j2-debug.xml";
pub const LOG4J2_LEGACY_DEBUG_NAME: &str = "log4j2-legacy-debug.xml";

pub const JAVA_WRAPPER: &[u8] = include_bytes!("../../../resources/launch/java-wrapper.jar");
pub const LEGACY_FIX: &[u8] = include_bytes!("../../../resources/launch/legacyfix.jar");
pub const LWJGL_UNSAFE_AGENT: &[u8] =
    include_bytes!("../../../resources/launch/lwjgl-unsafe-agent.jar");
pub const LOG4J2_DEBUG: &[u8] = include_bytes!("../../../resources/launch/log4j2-debug.xml");
pub const LOG4J2_LEGACY_DEBUG: &[u8] =
    include_bytes!("../../../resources/launch/log4j2-legacy-debug.xml");
pub const MACOS_ARM_LIBRARIES: &str =
    include_str!("../../../resources/launch/macos-arm64-libraries.json");

/// 渲染器代理的版本，对照 CE 的 mesa-loader-windows。
pub const MESA_LOADER_VERSION: &str = "26.0.4";

pub fn mesa_loader_path(directory: &Path) -> PathBuf {
    directory
        .join("mesa-loader-windows")
        .join(MESA_LOADER_VERSION)
        .join("Loader.jar")
}

/// 下载渲染器代理；CE 的镜像没有校验值。
pub async fn ensure_mesa_loader(downloader: &Downloader, path: &Path) -> Result<(), String> {
    if path.is_file() {
        return Ok(());
    }
    let parent = path.parent().ok_or("Invalid renderer agent path")?;
    std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    let arch = match crate::utils::platform::host_architecture() {
        "x86" => "x86",
        "aarch64" => "arm64",
        _ => "x64",
    };
    let url = format!(
        "https://mirrors.cloud.tencent.com/nexus/repository/maven-public/org/glavo/mesa-loader-windows/{MESA_LOADER_VERSION}/mesa-loader-windows-{MESA_LOADER_VERSION}-{arch}.jar"
    );
    log::info!("Downloading renderer agent from {url}");
    let response = downloader
        .send_request(
            HttpMode::Auto,
            true,
            &url,
            None,
            Some(std::time::Duration::from_secs(120)),
        )
        .await
        .map_err(|error| error.to_string())?
        .error_for_status()
        .map_err(|error| error.to_string())?;
    let bytes = response.bytes().await.map_err(|error| error.to_string())?;
    if !(1024..=64 * 1024 * 1024).contains(&bytes.len()) {
        return Err("Invalid renderer agent size".into());
    }
    let temp = path.with_extension(format!("{}.tmp", std::process::id()));
    tokio::fs::write(&temp, &bytes)
        .await
        .map_err(|error| error.to_string())?;
    tokio::fs::rename(&temp, &path)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}
