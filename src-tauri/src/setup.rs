use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};

use serde::Serialize;

use crate::core::auth::Account;

pub mod constants {
    pub const DEFAULT_JAVA_LIST_CACHE_VERSION: i32 = 0;
    pub const LAUNCHER_NAME: &str = "PCL.Proto";
    pub const USER_AGENT: &str = "PCL-Community/PCL.Proto/0.5.0";
}

/// PCL global setup info, which user can modify
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PCLSetupInfo {
    java_list_cache_version: i32,
    theme: Theme,
    download_sourse: DownloadSource,
}

/// PCL theme
#[derive(serde::Serialize, serde::Deserialize)]
enum Theme {
    BlueLight,
    BlueDark,
}

#[derive(serde::Serialize, serde::Deserialize)]
enum DownloadSource {
    Official,
    BMCLApi,
}

/// PCL setup info default impl
impl Default for PCLSetupInfo {
    /// PCL setup info default impl
    fn default() -> Self {
        PCLSetupInfo {
            java_list_cache_version: constants::DEFAULT_JAVA_LIST_CACHE_VERSION,
            theme: Theme::BlueLight,
            download_sourse: DownloadSource::Official,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GameDir {
    name: String,
    path: PathBuf,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct AppState {
    pub java_runtimes: Vec<crate::core::java::JavaRuntime>,
    pub pcl_setup_info: crate::setup::PCLSetupInfo,
    pub account: crate::core::auth::Account,
    pub game_directories: Vec<GameDir>,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            java_runtimes: Vec::new(),
            pcl_setup_info: crate::setup::PCLSetupInfo::default(),
            account: Account::Offline {
                username: "AMagicPear".to_string(),
                uuid: "12345678-1234-1234-1234-123456789012".to_string(),
            },
            game_directories: Vec::new(),
        }
    }
}

/// config manager, for loading and saving config file
/// should be Singleton
/// ## Fields
/// * `config_path` - the path of config file
pub struct ConfigManager {
    config_path: PathBuf,
    config_dir: PathBuf,
    pub app_state: Arc<Mutex<AppState>>,
}

#[derive(Debug)]
pub enum ConfigManagerError {
    ConfigDirNotFound,
    ConfigFileNotFound,
    ConfigFileCorrupted,
}

static CONFIG_MANAGER: LazyLock<ConfigManager> = LazyLock::new(|| ConfigManager::new().unwrap());

impl ConfigManager {
    /// create a new config manager every time it launches
    /// and initialize the config file, for those who never use this launcher
    pub fn new() -> Result<Self, ConfigManagerError> {
        let config_dirs = directories::ProjectDirs::from("cc", "PCL Community", "PCL.Proto")
            .ok_or(ConfigManagerError::ConfigDirNotFound)?;
        let config_dir = config_dirs.config_dir();
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).map_err(|_| ConfigManagerError::ConfigDirNotFound)?;
        }
        let config_path = config_dir.join("config.json");
        let instance = Self {
            config_path,
            config_dir: config_dir.to_path_buf(),
            app_state: Arc::new(Mutex::new(AppState::default())),
        };
        if instance.config_path.exists() {
            instance.load()?;
        } else {
            instance.init()?;
            instance.save()?;
        }
        Ok(instance)
    }

    pub fn instance() -> &'static ConfigManager {
        &CONFIG_MANAGER
    }

    /// initialize the config file, for those who never use this launcher
    fn init(&self) -> Result<(), ConfigManagerError> {
        let game_dir = self.config_dir.join(".minecraft");
        if !game_dir.exists() {
            fs::create_dir_all(&game_dir).map_err(|_| ConfigManagerError::ConfigDirNotFound)?;
        }
        let mut state = self.app_state.lock().unwrap();
        state.game_directories.push(GameDir {
            name: "current".to_string(),
            path: game_dir,
        });
        Ok(())
    }

    /// load the config file
    fn load(&self) -> Result<(), ConfigManagerError> {
        let file = std::fs::File::open(&self.config_path)
            .map_err(|_| ConfigManagerError::ConfigFileNotFound)?;
        let mut reader = std::io::BufReader::new(file);
        let state_read: AppState = serde_json::from_reader(&mut reader)
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        let mut state = self.app_state.lock().unwrap();
        *state = state_read;
        Ok(())
    }

    /// save the config file
    pub fn save(&self) -> Result<(), ConfigManagerError> {
        let file = std::fs::File::create(&self.config_path)
            .map_err(|_| ConfigManagerError::ConfigFileNotFound)?;
        let mut writer = std::io::BufWriter::new(file);
        let state = self.app_state.lock().unwrap();
        serde_json::to_writer_pretty(&mut writer, &*state)
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        writer
            .flush()
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        Ok(())
    }
}

#[test]
fn test_config_manager() {
    let config_manager = ConfigManager::instance();
    config_manager.load().unwrap();
    config_manager.save().unwrap();
}
