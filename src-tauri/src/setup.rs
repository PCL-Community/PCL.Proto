use std::{fs, path::PathBuf};

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
                username: "PCL.Proto-Test".to_string(),
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
}

impl ConfigManager {
    /// create a new config manager every time it launches
    pub fn new() -> Result<Self, ()> {
        let config_dirs =
            directories::ProjectDirs::from("cc", "PCL Community", "PCL.Proto").ok_or(())?;
        let config_dir = config_dirs.config_dir();
        fs::create_dir_all(config_dir).map_err(|_| ())?;
        let config_path = config_dir.join("config.json");
        Ok(Self {
            config_path,
            config_dir: config_dir.to_path_buf(),
        })
    }

    /// initialize the config file, for those who never use this launcher
    fn init(&self, state: &mut AppState) -> Result<(), ()> {
        let game_dir = self.config_dir.join(".minecraft");
        if !game_dir.exists() {
            fs::create_dir_all(&game_dir).map_err(|_| ())?;
        }
        state.game_directories.push(GameDir {
            name: "current".to_string(),
            path: game_dir,
        });
        self.save(state)?;
        Ok(())
    }

    /// load the config file or initialize it if it doesn't exist
    pub fn load(&self) -> Result<AppState, ()> {
        if !self.config_path.exists() {
            let mut state = AppState::default();
            self.init(&mut state)?;
            return Ok(state);
        }
        let file = std::fs::File::open(&self.config_path).map_err(|_| ())?;
        let mut reader = std::io::BufReader::new(file);
        let state: AppState = serde_json::from_reader(&mut reader).map_err(|_| ())?;
        Ok(state)
    }

    /// save the config file
    pub fn save(&self, state: &AppState) -> Result<(), ()> {
        let file = std::fs::File::create(&self.config_path).map_err(|_| ())?;
        let mut writer = std::io::BufWriter::new(file);
        serde_json::to_writer(&mut writer, state).map_err(|_| ())?;
        Ok(())
    }
}

#[test]
fn test_config_manager() {
    let config_manager = ConfigManager::new().unwrap();
    let state = config_manager.load();
    // config_manager.save(&state).unwrap();
}
