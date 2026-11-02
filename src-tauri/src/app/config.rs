pub use crate::app::metadata::constants;
pub use crate::app::states::{AppState, WindowMode, WindowSettings};

use tokio::sync::Mutex;

use crate::{
    app::config::constants::USER_AGENT,
    io::net::http::api_client::{ApiProvider, MinecraftApiClient},
    minecraft::instance::{McInstance, folder::McFolder},
};
use std::{
    fs,
    io::Write,
    path::PathBuf,
    sync::{Arc, LazyLock},
};

#[tauri::command]
pub async fn get_window_settings() -> Option<WindowSettings> {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .window_settings
        .clone()
}

#[tauri::command]
pub async fn get_launcher_visibility() -> crate::app::visibility::LauncherVisibility {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .launcher_visibility
}

#[tauri::command]
pub async fn set_launcher_visibility(
    mode: crate::app::visibility::LauncherVisibility,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            state.launcher_visibility = mode;
            Ok(())
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn set_window_settings(settings: WindowSettings) -> Result<WindowSettings, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update_window_settings(settings)
    })
    .await
    .map_err(|error| error.to_string())?
}

impl ConfigManager {
    pub fn ensure_instance_isolation(&self, path: &std::path::Path) -> Result<bool, String> {
        {
            let state = self.app_state.blocking_lock();
            crate::minecraft::java::preference::current_instance(&state, path)
                .map_err(|error| error.to_string())?;
            if let Some(value) = state.instance_isolation.get(path) {
                return Ok(*value);
            }
        }
        self.update(|state| crate::minecraft::instance::isolation::ensure(state, path))
    }

    fn update_window_settings(&self, settings: WindowSettings) -> Result<WindowSettings, String> {
        settings.validate()?;
        self.update(|state| {
            state.window_settings = Some(settings.clone());
            Ok(settings)
        })
    }
}

#[tauri::command]
pub async fn get_log_max_lines() -> Option<u32> {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .log_max_lines
}

#[tauri::command]
pub async fn set_log_max_lines(lines: u32) -> Result<u32, String> {
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update_log_max_lines(lines)
    })
    .await
    .map_err(|e| e.to_string())?
}

impl ConfigManager {
    fn update_log_max_lines(&self, lines: u32) -> Result<u32, String> {
        if lines != 0 && !(50..=2000).contains(&lines) {
            return Err("Invalid log line limit".into());
        }
        self.update(|state| {
            state.log_max_lines = Some(lines);
            Ok(lines)
        })
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
    pub api_client: MinecraftApiClient,
    pub pcl_identifier: String,
}

#[derive(Debug)]
pub enum ConfigManagerError {
    ConfigDirNotFound,
    ConfigFileNotFound,
    ConfigFileCorrupted,
    IdentifierFailure,
    ApiClientBuildError,
}

pub static CONFIG_MANAGER: LazyLock<Result<ConfigManager, ConfigManagerError>> =
    LazyLock::new(ConfigManager::new);

impl ConfigManager {
    /// create a new config manager every time it launches
    /// and initialize the config file, for those who never use this launcher
    fn new() -> Result<Self, ConfigManagerError> {
        let config_dirs = directories::ProjectDirs::from("pcl-community", "pcl", "proto")
            .ok_or(ConfigManagerError::ConfigDirNotFound)?;
        let config_dir = config_dirs.config_dir();
        #[cfg(test)]
        let test_config_dir = std::env::var_os("PCL_TEST_CONFIG_ROOT").map(PathBuf::from);
        #[cfg(test)]
        let config_dir = test_config_dir.as_deref().unwrap_or(config_dir);
        if !config_dir.exists() {
            fs::create_dir_all(config_dir).map_err(|_| ConfigManagerError::ConfigDirNotFound)?;
        }
        let config_path = config_dir.join("config.json");
        let identifier_path = config_dir.join("pcl_identifier.txt");
        let pcl_identifier = if identifier_path.exists() {
            fs::read_to_string(identifier_path)
                .map_err(|_| ConfigManagerError::IdentifierFailure)?
        } else {
            let pcl_identifier = crate::utils::secret::Identify::launcher_id()
                .map_err(|_| ConfigManagerError::IdentifierFailure)?;
            fs::write(identifier_path, pcl_identifier.clone())
                .map_err(|_| ConfigManagerError::IdentifierFailure)?;
            pcl_identifier
        };
        let instance = Self {
            config_path,
            config_dir: config_dir.to_path_buf(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(
                reqwest::Client::builder()
                    .user_agent(USER_AGENT)
                    .build()
                    .map_err(|_| ConfigManagerError::ApiClientBuildError)?,
                &ApiProvider::default(),
            ),
            pcl_identifier,
        };
        instance.load_or_init()?;
        // The API client is built before the proxy policy is loaded.
        instance
            .api_client
            .rebuild_client()
            .map_err(|_| ConfigManagerError::ApiClientBuildError)?;
        let download_settings = instance
            .app_state
            .blocking_lock()
            .download_settings
            .clone()
            .unwrap_or_default();
        crate::io::download::resources::DOWNLOAD_RESOURCES
            .configure(download_settings)
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        log::debug!("launched config at {:?}", instance.config_path);
        Ok(instance)
    }

    pub fn instance() -> &'static ConfigManager {
        CONFIG_MANAGER
            .as_ref()
            .expect("config manager failed to initialize")
    }

    /// Directory holding the config file and extracted launcher resources.
    pub fn config_dir(&self) -> &std::path::Path {
        &self.config_dir
    }

    pub fn update_proxy_settings(
        &self,
        settings: crate::io::net::proxy::ProxySettings,
    ) -> Result<crate::io::net::proxy::ProxySettings, String> {
        settings.validate()?;
        // Validate the resulting client before committing the setting.
        crate::io::net::proxy::build_proxy_client(&settings)?;
        self.update(|state| {
            state.proxy = settings.clone();
            Ok(settings.clone())
        })?;
        crate::io::net::proxy::apply(&settings)?;
        self.api_client.rebuild_client()?;
        Ok(settings)
    }

    fn load_or_init(&self) -> Result<(), ConfigManagerError> {
        match self.load() {
            Ok(()) => Ok(()),
            Err(ConfigManagerError::ConfigFileNotFound) => {
                match fs::symlink_metadata(&self.config_path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        self.init()?;
                        self.save()
                    }
                    _ => Err(ConfigManagerError::ConfigFileCorrupted),
                }
            }
            Err(error) => Err(error),
        }
    }

    /// initialize the config file, for those who never use this launcher
    fn init(&self) -> Result<(), ConfigManagerError> {
        let game_dir = self.config_dir.join(".minecraft");
        if !game_dir.exists() {
            fs::create_dir_all(&game_dir).map_err(|_| ConfigManagerError::ConfigDirNotFound)?;
        }
        let mut state = self.app_state.blocking_lock();
        state.active_repo_path = game_dir.clone();
        state.repositories.push(McFolder::new("Default", game_dir));
        // CE 一致：首次运行不预置账号，由用户自行登录。
        Ok(())
    }

    /// load the config file
    /// TODO: Partial config support
    fn load(&self) -> Result<(), ConfigManagerError> {
        let file = std::fs::File::open(&self.config_path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ConfigManagerError::ConfigFileNotFound
            } else {
                ConfigManagerError::ConfigFileCorrupted
            }
        })?;
        let mut reader = std::io::BufReader::new(file);
        let state_read: AppState = serde_json::from_reader(&mut reader)
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        if state_read
            .log_max_lines
            .is_some_and(|lines| lines != 0 && !(50..=2000).contains(&lines))
        {
            return Err(ConfigManagerError::ConfigFileCorrupted);
        }
        if state_read
            .window_settings
            .as_ref()
            .is_some_and(|settings| settings.validate().is_err())
        {
            return Err(ConfigManagerError::ConfigFileCorrupted);
        }
        if state_read
            .memory_settings
            .as_ref()
            .is_some_and(|settings| settings.validate().is_err())
            || state_read
                .instance_memory_settings
                .values()
                .any(|settings| settings.validate().is_err())
        {
            return Err(ConfigManagerError::ConfigFileCorrupted);
        }
        if state_read
            .memory_custom
            .as_ref()
            .is_some_and(|custom| custom.validate().is_err())
            || state_read
                .instance_memory_custom
                .values()
                .any(|custom| custom.validate().is_err())
        {
            return Err(ConfigManagerError::ConfigFileCorrupted);
        }
        if state_read.renderer > 3
            || state_read.proxy.validate().is_err()
            || state_read
                .instance_launch_settings
                .values()
                .any(|settings| settings.validate().is_err())
        {
            return Err(ConfigManagerError::ConfigFileCorrupted);
        }
        crate::io::net::proxy::apply(&state_read.proxy)
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        let mut state = self.app_state.blocking_lock();
        *state = state_read;
        // update the api provider
        self.api_client
            .switch_provider(&state.launcher_setup.api_provider);
        self.api_client
            .rebuild_client()
            .map_err(|_| ConfigManagerError::ApiClientBuildError)?;
        Ok(())
    }

    /// save the config file
    pub fn save(&self) -> Result<(), ConfigManagerError> {
        let state = self.app_state.blocking_lock();
        save_config_atomically(&self.config_path, &*state)
    }

    pub fn update<T>(
        &self,
        change: impl FnOnce(&mut AppState) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut state = self.app_state.blocking_lock();
        let mut candidate = state.clone();
        let result = change(&mut candidate)?;
        save_config_atomically(&self.config_path, &candidate).map_err(|e| format!("{e:?}"))?;
        *state = candidate;
        Ok(result)
    }

    pub fn select_repository(&self, index: usize) -> Result<(), String> {
        self.update(|state| {
            state.active_repo_path = state
                .repositories
                .get(index)
                .ok_or("Minecraft folder index is out of range")?
                .path
                .clone();
            Ok(())
        })
    }

    pub fn add_repository(
        &self,
        name: &str,
        path: &std::path::Path,
    ) -> Result<Vec<McFolder>, String> {
        let name = name.trim();
        if name.is_empty() || !path.is_absolute() || !path.is_dir() {
            return Err("Repository requires a name and an existing absolute directory".into());
        }
        let path = path.canonicalize().map_err(|error| error.to_string())?;
        self.update(|state| {
            if state.repositories.iter().any(|repo| {
                repo.name == name
                    || repo.path == path
                    || repo
                        .path
                        .canonicalize()
                        .is_ok_and(|existing| existing == path)
            }) {
                return Err("Repository name or path already exists".into());
            }
            state.repositories.push(McFolder::new(name, path));
            Ok(state.repositories.clone())
        })
    }

    pub fn select_instance(
        &self,
        index: usize,
        repo_path: &std::path::Path,
        instance: McInstance,
    ) -> Result<(), String> {
        self.update(|state| {
            let repo = state
                .repositories
                .get(index)
                .ok_or("Minecraft folder index is out of range")?;
            if repo.path != repo_path || instance.global_dir != repo.path {
                return Err("Minecraft folder changed; try again".into());
            }
            state.active_repo_path = repo.path.clone();
            state.active_game_instance = Some(Arc::new(instance));
            Ok(())
        })
    }

    pub fn update_download_settings(
        &self,
        settings: crate::io::download::resources::DownloadSettings,
    ) -> Result<crate::io::download::resources::DownloadSettings, String> {
        settings.validate()?;
        let mut state = self.app_state.blocking_lock();
        let mut candidate = state.clone();
        candidate.download_settings = Some(settings.clone());
        save_config_atomically(&self.config_path, &candidate).map_err(|e| format!("{e:?}"))?;
        *state = candidate;
        // 副作用放在提交之后：失败时磁盘与内存一致，用户重试即可恢复。
        crate::io::download::resources::DOWNLOAD_RESOURCES.configure(settings.clone())?;
        Ok(settings)
    }

    pub fn select_installed_instance(
        &self,
        repo_path: &std::path::Path,
        instance: McInstance,
    ) -> Result<bool, String> {
        let mut state = self.app_state.blocking_lock();
        if !state
            .download_settings
            .as_ref()
            .and_then(|s| s.auto_select_instance)
            .unwrap_or(false)
            || state.active_repo_path != repo_path
            || instance.global_dir != repo_path
            || !state.repositories.iter().any(|repo| repo.path == repo_path)
        {
            return Ok(false);
        }
        let mut candidate = state.clone();
        candidate.active_game_instance = Some(Arc::new(instance));
        save_config_atomically(&self.config_path, &candidate).map_err(|e| format!("{e:?}"))?;
        *state = candidate;
        Ok(true)
    }

    #[inline]
    pub fn identifier_path(&self) -> PathBuf {
        self.config_dir.join("pcl_identifier.txt")
    }

    pub fn java_runtime_dir(&self) -> PathBuf {
        self.config_dir.join("runtime")
    }
}

fn save_config_atomically(
    path: &std::path::Path,
    value: &impl serde::Serialize,
) -> Result<(), ConfigManagerError> {
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)
            .map_err(|_| ConfigManagerError::ConfigFileNotFound)?;
        let mut writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, value)
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        writer
            .flush()
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        writer
            .get_ref()
            .sync_all()
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        drop(writer);
        fs::rename(&temporary, path).map_err(|_| ConfigManagerError::ConfigFileCorrupted)
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}

#[test]
fn test_config_manager() {
    // Keep the test away from the real per-user config and from the global
    // download resources configured by the singleton.
    let root = std::env::temp_dir().join(format!("pcl-config-{}", uuid::Uuid::new_v4()));
    fs::create_dir_all(&root).unwrap();
    let config_path = root.join("config.json");
    let config_manager = ConfigManager {
        config_path: config_path.clone(),
        config_dir: root.clone(),
        app_state: Arc::new(Mutex::new(AppState::default())),
        api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
        pcl_identifier: String::new(),
    };
    config_manager.save().unwrap();
    config_manager.load().unwrap();
    assert!(config_path.is_file());
    fs::remove_dir_all(root).unwrap();
}

#[cfg(test)]
mod java_config_tests {
    use super::*;
    use serde::ser::Error;
    #[test]
    fn isolation_initialization_is_transactional_and_persists_per_directory() {
        let root =
            std::env::temp_dir().join(format!("pcl-isolation-config-{}", uuid::Uuid::new_v4()));
        let path = root.join("versions/first");
        fs::create_dir_all(&path).unwrap();
        fs::write(path.join("first.json"), r#"{"id":"same","type":"release"}"#).unwrap();
        fs::write(path.join("first.jar"), b"jar").unwrap();
        let instance =
            McInstance::from_version_folder(&path, &McFolder::new("test", root.clone())).unwrap();
        let config_path = root.join("config.json");
        let manager = ConfigManager {
            config_path: config_path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState {
                active_game_instance: Some(Arc::new(instance)),
                isolation_default: crate::minecraft::instance::isolation::IsolationDefault::Off,
                ..Default::default()
            })),
            api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
            pcl_identifier: String::new(),
        };
        fs::create_dir(&config_path).unwrap();
        assert!(manager.ensure_instance_isolation(&path).is_err());
        assert!(
            manager
                .app_state
                .blocking_lock()
                .instance_isolation
                .is_empty()
        );
        fs::remove_dir(&config_path).unwrap();
        assert!(!manager.ensure_instance_isolation(&path).unwrap());
        let saved: AppState = serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
        assert_eq!(saved.instance_isolation.get(&path), Some(&false));
        manager
            .update(|state| {
                state.isolation_default =
                    crate::minecraft::instance::isolation::IsolationDefault::All;
                Ok(())
            })
            .unwrap();
        assert!(!manager.ensure_instance_isolation(&path).unwrap());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn launch_arguments_validate_and_commit_per_instance_transactionally() {
        use crate::minecraft::launch::arguments::{LaunchArguments, update};
        let root =
            std::env::temp_dir().join(format!("pcl-argument-config-{}", uuid::Uuid::new_v4()));
        let repo = McFolder::new("test", root.clone());
        let mut instances = Vec::new();
        for name in ["one", "two"] {
            let path = root.join("versions").join(name);
            fs::create_dir_all(&path).unwrap();
            fs::write(
                path.join(format!("{name}.json")),
                r#"{"id":"same","type":"release"}"#,
            )
            .unwrap();
            fs::write(path.join(format!("{name}.jar")), b"jar").unwrap();
            instances.push(McInstance::from_version_folder(&path, &repo).unwrap());
        }
        let config_path = root.join("config.json");
        let manager = ConfigManager {
            config_path: config_path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState {
                active_game_instance: Some(Arc::new(instances[0].clone())),
                ..Default::default()
            })),
            api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
            pcl_identifier: String::new(),
        };
        let valid = LaunchArguments {
            jvm: "-Dfirst=true".into(),
            ..Default::default()
        };
        let first = instances[0].directory.clone();
        let second = instances[1].directory.clone();
        assert!(
            manager
                .update(|state| update(state, Some(&second), valid.clone()))
                .is_err()
        );
        assert!(
            manager
                .update(|state| update(
                    state,
                    None,
                    LaunchArguments {
                        game: "'invalid".into(),
                        ..Default::default()
                    }
                ))
                .is_err()
        );
        fs::create_dir(&config_path).unwrap();
        assert!(
            manager
                .update(|state| update(state, Some(&first), valid.clone()))
                .is_err()
        );
        assert!(
            manager
                .app_state
                .blocking_lock()
                .instance_launch_arguments
                .is_empty()
        );
        fs::remove_dir(&config_path).unwrap();
        manager
            .update(|state| update(state, Some(&first), valid.clone()))
            .unwrap();
        manager
            .update(|state| {
                state.active_game_instance = Some(Arc::new(instances[1].clone()));
                Ok(())
            })
            .unwrap();
        let other = LaunchArguments {
            game: "--demo".into(),
            ..Default::default()
        };
        manager
            .update(|state| update(state, Some(&second), other.clone()))
            .unwrap();
        let saved: AppState = serde_json::from_slice(&fs::read(&config_path).unwrap()).unwrap();
        assert_eq!(saved.instance_launch_arguments.get(&first), Some(&valid));
        assert_eq!(saved.instance_launch_arguments.get(&second), Some(&other));
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn window_settings_validate_and_preserve_failed_save() {
        let root =
            std::env::temp_dir().join(format!("pcl-window-settings-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let manager = ConfigManager {
            config_path: root.join("config.json"),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
            pcl_identifier: String::new(),
        };
        let custom = WindowSettings {
            mode: WindowMode::Custom,
            width: 1280,
            height: 720,
        };
        assert_eq!(
            manager.update_window_settings(custom.clone()).unwrap(),
            custom
        );
        let default = WindowSettings {
            mode: WindowMode::Default,
            width: 854,
            height: 480,
        };
        assert!(
            manager
                .update_window_settings(WindowSettings {
                    width: 99,
                    ..default.clone()
                })
                .is_err()
        );
        let restored: AppState =
            serde_json::from_slice(&fs::read(&manager.config_path).unwrap()).unwrap();
        assert_eq!(restored.window_settings, Some(custom.clone()));
        fs::remove_file(&manager.config_path).unwrap();
        fs::create_dir(&manager.config_path).unwrap();
        assert!(manager.update_window_settings(default).is_err());
        assert_eq!(
            manager.app_state.blocking_lock().window_settings,
            Some(custom)
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn initialization_preserves_invalid_config_and_creates_missing_config() {
        let root =
            std::env::temp_dir().join(format!("pcl-config-init-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let path = root.join("config.json");
        let manager = ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
            pcl_identifier: String::new(),
        };
        fs::write(&path, b"invalid json").unwrap();
        assert!(matches!(
            manager.load_or_init(),
            Err(ConfigManagerError::ConfigFileCorrupted)
        ));
        assert_eq!(fs::read(&path).unwrap(), b"invalid json");
        fs::remove_file(&path).unwrap();
        manager.load_or_init().unwrap();
        assert!(serde_json::from_slice::<AppState>(&fs::read(&path).unwrap()).is_ok());
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn failed_saves_preserve_previous_config_and_successful_saves_replace_it() {
        let root =
            std::env::temp_dir().join(format!("pcl-java-config-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let path = root.join("config.json");
        fs::write(&path, b"previous config").unwrap();
        struct CannotSerialize;
        impl serde::Serialize for CannotSerialize {
            fn serialize<S: serde::Serializer>(&self, _: S) -> Result<S::Ok, S::Error> {
                Err(S::Error::custom("fixture failure"))
            }
        }
        assert!(save_config_atomically(&path, &CannotSerialize).is_err());
        assert_eq!(fs::read(&path).unwrap(), b"previous config");
        let mut state = AppState::default();
        state.launcher_setup.selected_java = Some(PathBuf::from("/test/jdk/bin/java"));
        state.java_preferences.insert(
            PathBuf::from("/test/game"),
            crate::minecraft::java::preference::InstanceJavaSettings {
                preference: crate::minecraft::java::preference::JavaPreference::Auto,
                ignore_compatibility: true,
            },
        );
        save_config_atomically(&path, &state).unwrap();
        let restored: AppState = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            restored.launcher_setup.selected_java,
            state.launcher_setup.selected_java
        );
        assert!(restored.java_preferences[&PathBuf::from("/test/game")].ignore_compatibility);
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn updates_publish_only_after_save_and_preserve_concurrent_changes() {
        let root = std::env::temp_dir().join(format!("pcl-config-update-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let path = root.join("config.json");
        let manager = Arc::new(ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
            pcl_identifier: String::new(),
        });
        fs::create_dir(&path).unwrap();
        assert!(
            manager
                .update(|state| {
                    state.launcher_setup.max_memory = 4096;
                    Ok(())
                })
                .is_err()
        );
        assert_eq!(
            manager.app_state.blocking_lock().launcher_setup.max_memory,
            2048
        );
        fs::remove_dir(&path).unwrap();
        let threads: Vec<_> = (0..8)
            .map(|i| {
                let manager = manager.clone();
                std::thread::spawn(move || {
                    manager
                        .update(|state| {
                            state
                                .java_preferences
                                .insert(PathBuf::from(format!("/game/{i}")), Default::default());
                            Ok(())
                        })
                        .unwrap()
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        let saved: AppState = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved.java_preferences.len(), 8);
        assert_eq!(manager.app_state.blocking_lock().java_preferences.len(), 8);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn repository_and_instance_changes_are_transactional() {
        let root =
            std::env::temp_dir().join(format!("pcl-repository-settings-{}", uuid::Uuid::new_v4()));
        let repo = root.join("game");
        let version = repo.join("versions/test");
        fs::create_dir_all(&version).unwrap();
        fs::write(version.join("test.json"), r#"{"id":"test"}"#).unwrap();
        fs::write(version.join("test.jar"), b"jar").unwrap();
        let path = root.join("config.json");
        let manager = ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
            pcl_identifier: String::new(),
        };
        manager.add_repository("Games", &repo).unwrap();
        assert!(manager.add_repository("Other", &repo.join(".")).is_err());
        let saved_repo = manager.app_state.blocking_lock().repositories[0].clone();
        let instance = McInstance::from_version_folder(&version, &saved_repo).unwrap();
        assert!(manager.select_instance(0, &root, instance.clone()).is_err());
        manager
            .select_instance(0, &saved_repo.path, instance.clone())
            .unwrap();
        let saved: AppState = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved.active_game_instance.unwrap().id, "test");
        assert_eq!(saved.active_repo_path, saved_repo.path);
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(manager.select_repository(0).is_err());
        assert!(manager.add_repository("Again", &root).is_err());
        assert!(
            manager
                .select_instance(0, &saved_repo.path, instance)
                .is_err()
        );
        let state = manager.app_state.blocking_lock();
        assert_eq!(state.repositories.len(), 1);
        assert_eq!(state.active_game_instance.as_ref().unwrap().id, "test");
        drop(state);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn download_settings_validate_and_failed_save_preserves_state() {
        use crate::io::download::resources::{DOWNLOAD_RESOURCES, DownloadSettings};
        let previous = DOWNLOAD_RESOURCES.settings();
        let root =
            std::env::temp_dir().join(format!("pcl-download-settings-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let path = root.join("config.json");
        let manager = ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
            pcl_identifier: String::new(),
        };
        let first = DownloadSettings {
            max_connections: 17,
            auto_select_instance: None,
            ..Default::default()
        };
        let second = DownloadSettings {
            max_connections: 32,
            auto_select_instance: Some(false),
            ..Default::default()
        };
        assert_eq!(
            manager
                .update_download_settings(first)
                .unwrap()
                .max_connections,
            17
        );
        assert_eq!(
            manager
                .update_download_settings(second.clone())
                .unwrap()
                .max_connections,
            32
        );
        let saved = fs::read(&path).unwrap();
        let restored: AppState = serde_json::from_slice(&saved).unwrap();
        assert_eq!(restored.download_settings.unwrap().max_connections, 32);
        assert_eq!(
            manager
                .app_state
                .blocking_lock()
                .download_settings
                .as_ref()
                .unwrap()
                .auto_select_instance,
            Some(false)
        );
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(manager.update_download_settings(second).is_err());
        assert_eq!(
            manager
                .app_state
                .blocking_lock()
                .download_settings
                .as_ref()
                .unwrap()
                .max_connections,
            32
        );
        assert_eq!(DOWNLOAD_RESOURCES.settings().max_connections, 32);
        DOWNLOAD_RESOURCES.configure(previous).unwrap();
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn automatic_instance_selection_respects_settings_and_repository() {
        let root = std::env::temp_dir().join(format!("pcl-auto-select-{}", uuid::Uuid::new_v4()));
        let version = root.join("versions/test");
        fs::create_dir_all(&version).unwrap();
        fs::write(version.join("test.json"), r#"{"id":"test"}"#).unwrap();
        fs::write(version.join("test.jar"), b"jar").unwrap();
        let repo = McFolder::new("test", root.clone());
        let instance = McInstance::from_version_folder(&version, &repo).unwrap();
        let manager = ConfigManager {
            config_path: root.join("config.json"),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState {
                repositories: vec![repo],
                active_repo_path: root.clone(),
                download_settings: Some(crate::io::download::resources::DownloadSettings {
                    auto_select_instance: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            })),
            api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
            pcl_identifier: String::new(),
        };
        assert!(
            !manager
                .select_installed_instance(&root, instance.clone())
                .unwrap()
        );
        manager
            .app_state
            .blocking_lock()
            .download_settings
            .as_mut()
            .unwrap()
            .auto_select_instance = Some(true);
        assert!(
            !manager
                .select_installed_instance(&root.join("other"), instance.clone())
                .unwrap()
        );
        assert!(
            manager
                .select_installed_instance(&root, instance.clone())
                .unwrap()
        );
        let saved: AppState =
            serde_json::from_slice(&fs::read(&manager.config_path).unwrap()).unwrap();
        assert_eq!(saved.active_game_instance.unwrap().id, "test");
        fs::remove_file(&manager.config_path).unwrap();
        fs::create_dir(&manager.config_path).unwrap();
        assert!(manager.select_installed_instance(&root, instance).is_err());
        assert_eq!(
            manager
                .app_state
                .blocking_lock()
                .active_game_instance
                .as_ref()
                .unwrap()
                .id,
            "test"
        );
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn log_limit_validation_and_failed_save_preserve_state() {
        let root = std::env::temp_dir().join(format!("pcl-log-settings-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let path = root.join("config.json");
        let manager = ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(reqwest::Client::new(), &ApiProvider::default()),
            pcl_identifier: String::new(),
        };
        assert_eq!(manager.update_log_max_lines(500).unwrap(), 500);
        assert_eq!(manager.update_log_max_lines(100).unwrap(), 100);
        assert!(manager.update_log_max_lines(2001).is_err());
        fs::remove_file(&path).unwrap();
        fs::create_dir(&path).unwrap();
        assert!(manager.update_log_max_lines(0).is_err());
        assert_eq!(manager.app_state.blocking_lock().log_max_lines, Some(100));
        fs::remove_dir_all(root).unwrap();
    }
}
