//! 本应用的辅助资源路径。
//! macOS 使用包内只读资源，其它平台由 core 按需准备默认资源。

use std::path::{Path, PathBuf};

use pcl_core::minecraft::{
    installer::forgelike,
    launch::{
        launcher::{GameLaunchError, LaunchHelpers},
        resources,
    },
};

/// 从可执行文件路径推导 `.app` 包的 `Contents/Resources`。
#[cfg(not(all(target_os = "macos", dev)))]
pub fn bundle_root(executable: &Path) -> Option<PathBuf> {
    let macos = executable.parent()?;
    let contents = macos.parent()?;
    (macos.file_name()? == "MacOS" && contents.file_name()? == "Contents")
        .then(|| contents.join("Resources"))
}

#[cfg(not(all(target_os = "macos", dev)))]
fn running_bundle() -> Option<PathBuf> {
    let executable = std::env::current_exe().ok()?.canonicalize().ok()?;
    bundle_root(&executable)
}

pub struct Assets {
    directory: PathBuf,
    bundled_directory: Option<PathBuf>,
}

impl Assets {
    pub fn new(config_dir: &Path) -> Self {
        #[cfg(all(target_os = "macos", dev))]
        let bundled_directory = Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../crates/pcl-core/resources/launch"),
        );
        #[cfg(not(all(target_os = "macos", dev)))]
        let bundled_directory = running_bundle().map(|root| root.join("launcher"));
        Self {
            directory: config_dir.join("resources"),
            bundled_directory,
        }
    }

    pub fn forge_installer(&self) -> Option<PathBuf> {
        #[cfg(all(target_os = "macos", dev))]
        return Some(
            PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                .join("../crates/pcl-core/resources/installer")
                .join(forgelike::BRIDGE_NAME),
        );
        #[cfg(not(all(target_os = "macos", dev)))]
        self.bundled_directory
            .as_ref()
            .map(|directory| directory.join(forgelike::BRIDGE_NAME))
    }

    /// 包内文件覆盖 core 默认资源；下载的 Mesa loader 始终放在可写目录。
    pub fn launch_helpers(&self) -> Result<LaunchHelpers, GameLaunchError> {
        let bundled = |name: &str| {
            self.bundled_directory
                .as_ref()
                .map(|directory| directory.join(name))
        };
        let macos_arm_libraries = bundled(resources::MACOS_ARM_LIBRARIES_NAME)
            .map(std::fs::read_to_string)
            .transpose()?;
        Ok(LaunchHelpers {
            java_wrapper: bundled(resources::JAVA_WRAPPER_NAME),
            legacy_fix: bundled(resources::LEGACY_FIX_NAME),
            lwjgl_unsafe_agent: bundled(resources::LWJGL_UNSAFE_AGENT_NAME),
            log4j2_debug: bundled(resources::LOG4J2_DEBUG_NAME),
            log4j2_legacy_debug: bundled(resources::LOG4J2_LEGACY_DEBUG_NAME),
            mesa_loader: Some(resources::mesa_loader_path(&self.directory)),
            macos_arm_libraries,
        })
    }
}
