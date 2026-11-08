//! 启动兼容资源；默认按功能需要准备，也可由宿主打包后传入替代路径。

use crate::io::{download::Downloader, net::http::resources::HttpMode};
use std::path::{Path, PathBuf};

pub const JAVA_WRAPPER_NAME: &str = "java-wrapper.jar";
pub const LEGACY_FIX_NAME: &str = "legacyfix.jar";
pub const LWJGL_UNSAFE_AGENT_NAME: &str = "lwjgl-unsafe-agent.jar";
pub const LOG4J2_DEBUG_NAME: &str = "log4j2-debug.xml";
pub const LOG4J2_LEGACY_DEBUG_NAME: &str = "log4j2-legacy-debug.xml";

pub const MACOS_ARM_LIBRARIES_NAME: &str = "macos-arm64-libraries.json";

/// 外部资源宿主关闭 embedded-resources 后，辅助文件不会进入可执行文件。
pub fn embedded(name: &str) -> Option<&'static [u8]> {
    #[cfg(feature = "embedded-resources")]
    return match name {
        JAVA_WRAPPER_NAME => Some(include_bytes!("../../../resources/launch/java-wrapper.jar")),
        LEGACY_FIX_NAME => Some(include_bytes!("../../../resources/launch/legacyfix.jar")),
        LWJGL_UNSAFE_AGENT_NAME => Some(include_bytes!(
            "../../../resources/launch/lwjgl-unsafe-agent.jar"
        )),
        LOG4J2_DEBUG_NAME => Some(include_bytes!("../../../resources/launch/log4j2-debug.xml")),
        LOG4J2_LEGACY_DEBUG_NAME => Some(include_bytes!(
            "../../../resources/launch/log4j2-legacy-debug.xml"
        )),
        super::super::installer::forgelike::BRIDGE_NAME => Some(include_bytes!(
            "../../../resources/installer/forge-installer.jar"
        )),
        MACOS_ARM_LIBRARIES_NAME => Some(include_bytes!(
            "../../../resources/launch/macos-arm64-libraries.json"
        )),
        _ => None,
    };
    #[cfg(not(feature = "embedded-resources"))]
    {
        let _ = name;
        None
    }
}

/// 优先使用宿主提供的文件；缺失时明确报错，不能生成空 JAR 或回退到旧缓存。
pub fn helper_path(
    supplied: Option<&Path>,
    directory: &Path,
    name: &str,
) -> std::io::Result<PathBuf> {
    if let Some(path) = supplied {
        let metadata = std::fs::metadata(path).map_err(|error| {
            std::io::Error::new(
                error.kind(),
                format!("Missing resource {}: {error}", path.display()),
            )
        })?;
        if !metadata.is_file() || metadata.len() == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid resource: {}", path.display()),
            ));
        }
        return Ok(path.to_owned());
    }
    let bytes = embedded(name).ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("The host must provide {name}; embedded-resources is disabled"),
        )
    })?;
    let path = directory.join(name);
    crate::io::files::write_embedded(&path, bytes)?;
    Ok(path)
}

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
        .transport()
        .send_request(
            HttpMode::Auto,
            true,
            &crate::io::net::http::RequestTarget::new(url),
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
