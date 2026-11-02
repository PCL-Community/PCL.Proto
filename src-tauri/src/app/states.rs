//! Persisted launcher state, mirroring `PCL.Core.App.States` groups.

use crate::{
    app::metadata::constants,
    io::net::http::api_client::ApiProvider,
    minecraft::{
        instance::{McInstance, folder::McFolder},
        profile::McProfile,
    },
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

/// PCL global setup info, which user can modify
#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct LauncherSetup {
    setup_version: i32,
    theme: Theme,
    pub api_provider: ApiProvider,
    pub max_memory: usize,
    #[serde(default)]
    pub selected_java: Option<PathBuf>,
}

/// PCL theme
#[derive(Clone, serde::Serialize, serde::Deserialize)]
enum Theme {
    BlueLight,
    BlueDark,
}

/// PCL setup info default impl
impl Default for LauncherSetup {
    /// PCL setup info default impl
    fn default() -> Self {
        LauncherSetup {
            setup_version: constants::DEFAULT_JAVA_LIST_CACHE_VERSION,
            theme: Theme::BlueLight,
            api_provider: ApiProvider::Official,
            max_memory: 2048,
            selected_java: None,
        }
    }
}

/// 档案配置，对照 CE 的 `ProfileJson<McProfile>`。
///
/// `last_used` 是当前档案在 `profiles` 中的下标，`-1` 表示未选择；
/// 用下标表示当前档案，替代早期重复保存整份当前档案的做法。
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct ProfileConfig {
    pub last_used: i32,
    pub profiles: Vec<McProfile>,
}

impl Default for ProfileConfig {
    fn default() -> Self {
        Self {
            last_used: -1,
            profiles: Vec::new(),
        }
    }
}

impl ProfileConfig {
    /// 当前档案；未选择或下标越界时为 `None`（CE 的 `_NormalizeLastUsedUnsafe`）。
    pub fn current(&self) -> Option<&McProfile> {
        usize::try_from(self.last_used)
            .ok()
            .and_then(|index| self.profiles.get(index))
    }

    /// 选择档案下标，`None` 表示取消选择。
    pub fn select(&mut self, index: Option<usize>) {
        self.last_used = index.map_or(-1, |index| index as i32);
    }
}

/// Something that would be shared all over the app and may be displayed
#[derive(Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct AppState {
    pub java_runtimes: Vec<crate::minecraft::java::JavaEntry>,
    #[serde(default)]
    pub profile: ProfileConfig,
    #[serde(rename = "pcl_setup_info")]
    pub launcher_setup: LauncherSetup,
    pub repositories: Vec<McFolder>,
    pub active_game_instance: Option<Arc<McInstance>>,
    pub active_repo_path: PathBuf,
    #[serde(default)]
    pub java_preferences:
        HashMap<PathBuf, crate::minecraft::java::preference::InstanceJavaSettings>,
    #[serde(default)]
    pub download_settings: Option<crate::io::download::resources::DownloadSettings>,
    #[serde(default)]
    pub log_max_lines: Option<u32>,
    #[serde(default)]
    pub window_settings: Option<WindowSettings>,
    #[serde(default)]
    pub memory_settings: Option<crate::minecraft::launch::memory::MemorySettings>,
    #[serde(default)]
    pub instance_memory_settings:
        HashMap<PathBuf, crate::minecraft::launch::memory::MemorySettings>,
    #[serde(default)]
    pub memory_custom: Option<crate::minecraft::launch::memory::MemoryCustom>,
    #[serde(default)]
    pub instance_memory_custom: HashMap<PathBuf, crate::minecraft::launch::memory::MemoryCustom>,
    #[serde(default)]
    pub isolation_default: crate::minecraft::instance::isolation::IsolationDefault,
    #[serde(default)]
    pub instance_isolation: HashMap<PathBuf, bool>,
    #[serde(default)]
    pub launch_arguments: Option<crate::minecraft::launch::arguments::LaunchArguments>,
    #[serde(default)]
    pub instance_launch_arguments:
        HashMap<PathBuf, crate::minecraft::launch::arguments::LaunchArguments>,
    #[serde(default)]
    pub launcher_visibility: crate::app::visibility::LauncherVisibility,
    #[serde(default)]
    pub instance_launch_settings:
        HashMap<PathBuf, crate::minecraft::launch::settings::InstanceLaunchSettings>,
    #[serde(default)]
    pub renderer: u8,
    #[serde(default = "crate::minecraft::launch::settings::default_version_info")]
    pub version_info: String,
    #[serde(default = "default_true")]
    pub disable_jlw: bool,
    #[serde(default)]
    pub disable_legacy_fix: bool,
    #[serde(default)]
    pub disable_lwjgl_unsafe_agent: bool,
    #[serde(default)]
    pub proxy: crate::io::net::proxy::ProxySettings,
    #[serde(default)]
    pub instance_descriptions: HashMap<PathBuf, String>,
}

fn default_true() -> bool {
    true
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WindowMode {
    Default,
    Launcher,
    Custom,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct WindowSettings {
    pub mode: WindowMode,
    pub width: u32,
    pub height: u32,
}

impl WindowSettings {
    pub(crate) fn validate(&self) -> Result<(), String> {
        if !(100..=16384).contains(&self.width) || !(100..=16384).contains(&self.height) {
            return Err("Window dimensions must be between 100 and 16384 pixels".into());
        }
        Ok(())
    }
}
