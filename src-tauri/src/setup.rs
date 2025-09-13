use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock, Mutex},
};

use crate::core::{
    auth::Account, game::GameInstance, java::JavaRuntime, repository::GameRepository,
};

pub mod constants {
    pub const DEFAULT_JAVA_LIST_CACHE_VERSION: i32 = 0;
    pub const LAUNCHER_NAME: &str = "PCL.Proto";
    pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const USER_AGENT: &str = concat!("PCL-Community/PCL.Proto/", env!("CARGO_PKG_VERSION"));
}

/// PCL global setup info, which user can modify
#[derive(serde::Serialize, serde::Deserialize)]
pub struct PCLSetupInfo {
    java_list_cache_version: i32,
    theme: Theme,
    download_sourse: DownloadSource,
    pub max_memory: usize,
    pub default_java: Option<Arc<JavaRuntime>>,
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
            max_memory: 2048,
            default_java: None,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize, Default)]
pub struct AppState {
    pub java_runtimes: Vec<crate::core::java::JavaRuntime>,
    pub accounts: Vec<Account>,
    pub pcl_setup_info: crate::setup::PCLSetupInfo,
    pub active_account: Option<Arc<Account>>,
    pub repositories: Vec<GameRepository>,
    pub active_game_instance: Option<Arc<GameInstance>>,
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

pub static CONFIG_MANAGER: LazyLock<Option<ConfigManager>> = LazyLock::new(|| {
    let config_manager = ConfigManager::new();
    if config_manager.is_ok() {
        Some(config_manager.unwrap())
    } else {
        None
    }
});

impl ConfigManager {
    /// create a new config manager every time it launches
    /// and initialize the config file, for those who never use this launcher
    fn new() -> Result<Self, ConfigManagerError> {
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
        if !instance.config_path.exists()
            || !instance.config_path.is_file()
            || instance.load().is_err()
        {
            log::warn!("Unabled to load config file, try to init");
            instance.init()?;
            instance.save()?;
        }
        Ok(instance)
    }

    pub fn instance() -> &'static ConfigManager {
        CONFIG_MANAGER.as_ref().unwrap()
    }

    /// initialize the config file, for those who never use this launcher
    fn init(&self) -> Result<(), ConfigManagerError> {
        let game_dir = self.config_dir.join(".minecraft");
        if !game_dir.exists() {
            fs::create_dir_all(&game_dir).map_err(|_| ConfigManagerError::ConfigDirNotFound)?;
        }
        let mut state = self.app_state.lock().unwrap();
        // [WARN] Only for Debug!!
        // TODO: 后面去除下面的代码
        state
            .repositories
            .push(GameRepository::new("Default", game_dir));
        state.repositories.push(GameRepository::new(
            "HMCL",
            PathBuf::from("/Users/amagicpear/HMCL/.minecraft"),
        ));
        state.active_account = Some(Arc::new(Account::Offline {
            username: "AMagicPear".to_string(),
            uuid: "12345678-1234-1234-1234-123456789012".to_string(),
        }));
        state.pcl_setup_info.default_java =
            Some(Arc::new(JavaRuntime::try_from("/usr/bin/java").unwrap()));
        Ok(())
    }

    /// load the config file
    /// TODO: Partial config support
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
