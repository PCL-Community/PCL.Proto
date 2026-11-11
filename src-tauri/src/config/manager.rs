pub use super::states::{AppState, WindowSettings};

use super::link::{LinkSettings, LinkState};

#[cfg(unix)]
use std::os::unix::fs::OpenOptionsExt;
use tokio::sync::Mutex;

use pcl_core::{
    io::net::http::api_client::MinecraftApiClient,
    minecraft::instance::{
        McInstance,
        folder::{McFolder, McFolderType},
    },
};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{Arc, LazyLock},
};

pub async fn get_window_settings() -> Option<WindowSettings> {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .window_settings
        .clone()
}

pub async fn get_launcher_visibility() -> crate::window::LauncherVisibility {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .launcher_visibility
}

pub async fn set_launcher_visibility(
    mode: crate::window::LauncherVisibility,
) -> Result<(), String> {
    tokio::task::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            state.launcher_visibility = mode;
            Ok(())
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

pub async fn get_version_info() -> String {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .version_info
        .clone()
}

pub async fn set_version_info(value: String) -> Result<String, String> {
    tokio::task::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            crate::config::launch_settings::validate_info(&value)?;
            state.version_info = value.clone();
            Ok(value)
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

pub async fn set_window_settings(settings: WindowSettings) -> Result<WindowSettings, String> {
    tokio::task::spawn_blocking(move || ConfigManager::instance().update_window_settings(settings))
        .await
        .map_err(|error| error.to_string())?
}

impl ConfigManager {
    pub fn read_instance<T>(
        &self,
        path: &Path,
        read: impl FnOnce(&super::instance::InstanceConfig) -> Result<T, String>,
    ) -> Result<T, String> {
        let state = self.app_state.blocking_lock();
        super::java::active_instance(&state, path)?;
        read(&super::instance::InstanceConfig::load(path)?)
    }

    pub fn update_instance<T>(
        &self,
        path: &Path,
        change: impl FnOnce(&mut super::instance::InstanceConfig) -> Result<T, String>,
    ) -> Result<T, String> {
        let state = self.app_state.blocking_lock();
        super::java::active_instance(&state, path)?;
        super::instance::InstanceConfig::update(path, change)
    }

    pub fn ensure_instance_isolation(&self, path: &std::path::Path) -> Result<bool, String> {
        let state = self.app_state.blocking_lock();
        crate::config::java::active_instance(&state, path)?;
        crate::config::isolation::ensure(&state, path)
    }

    fn update_window_settings(&self, settings: WindowSettings) -> Result<WindowSettings, String> {
        settings.validate()?;
        self.update(|state| {
            state.window_settings = Some(settings.clone());
            Ok(settings)
        })
    }
}

pub async fn get_log_max_lines() -> Option<u32> {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .log_max_lines
}

pub async fn set_log_max_lines(lines: u32) -> Result<u32, String> {
    tokio::task::spawn_blocking(move || ConfigManager::instance().update_log_max_lines(lines))
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

pub async fn get_link_settings() -> LinkSettings {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .link_settings
        .clone()
}

pub async fn get_link_state() -> LinkState {
    ConfigManager::instance()
        .app_state
        .lock()
        .await
        .link_state
        .clone()
}

pub async fn set_link_settings(settings: LinkSettings) -> Result<LinkSettings, String> {
    tokio::task::spawn_blocking(move || ConfigManager::instance().update_link_settings(settings))
        .await
        .map_err(|error| error.to_string())?
}

/// CE 的「初始化」会重置整个 Link 组：设置、EULA、Natayark 令牌与公告缓存。
pub async fn reset_link_settings() -> Result<LinkSettings, String> {
    tokio::task::spawn_blocking(|| ConfigManager::instance().reset_link_group())
        .await
        .map_err(|error| error.to_string())?
}

pub async fn update_link_state<T, F>(change: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce(&mut LinkState) -> Result<T, String> + Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        ConfigManager::instance().update(move |state| change(&mut state.link_state))
    })
    .await
    .map_err(|error| error.to_string())?
}

impl ConfigManager {
    fn update_link_settings(&self, settings: LinkSettings) -> Result<LinkSettings, String> {
        settings.validate()?;
        self.update(|state| {
            state.link_settings = settings.clone();
            Ok(settings)
        })
    }

    fn reset_link_group(&self) -> Result<LinkSettings, String> {
        self.update(|state| {
            state.link_settings = LinkSettings::default();
            state.link_state = LinkState::default();
            Ok(state.link_settings.clone())
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

const CONFIG_VERSION: u64 = 1;

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
        let config_path = config_dir.join("config.yml");
        let identifier_path = config_dir.join("pcl_identifier.txt");
        let pcl_identifier = if identifier_path.exists() {
            fs::read_to_string(identifier_path)
                .map_err(|_| ConfigManagerError::IdentifierFailure)?
        } else {
            let pcl_identifier = crate::identity::Identify::launcher_id()
                .map_err(|_| ConfigManagerError::IdentifierFailure)?;
            fs::write(identifier_path, pcl_identifier.clone())
                .map_err(|_| ConfigManagerError::IdentifierFailure)?;
            pcl_identifier
        };
        let instance = Self {
            config_path,
            config_dir: config_dir.to_path_buf(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new({
                let (client, http1) = crate::config::network::clients(&Default::default())
                    .map_err(|_| ConfigManagerError::ApiClientBuildError)?;
                pcl_core::io::net::http::HttpTransport::new(client, http1)
            })
            .with_curseforge_api_key(crate::secrets::CURSEFORGE_API_KEY.as_deref()),
            pcl_identifier,
        };
        instance.load_or_init(
            super::paths::launcher_minecraft_directory().as_deref(),
            super::paths::official_minecraft_directory().as_deref(),
        )?;
        let download_settings = instance
            .app_state
            .blocking_lock()
            .download_settings
            .clone()
            .unwrap_or_default();
        instance
            .api_client
            .resources()
            .configure(download_settings.options())
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        log::info!(
            "Loaded configuration from {}",
            instance.config_path.display()
        );
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
        settings: crate::config::network::ProxySettings,
    ) -> Result<crate::config::network::ProxySettings, String> {
        settings.validate()?;
        // Validate the resulting client before committing the setting.
        let (client, http1) = crate::config::network::clients(&settings)?;
        let mut state = self.app_state.blocking_lock();
        let mut candidate = state.clone();
        candidate.proxy = settings.clone();
        save_config_atomically(&self.config_path, &candidate)
            .map_err(|error| format!("{error:?}"))?;
        *state = candidate;
        self.api_client.replace_clients(client, http1);
        Ok(settings)
    }

    fn load_or_init(
        &self,
        portable: Option<&Path>,
        official: Option<&Path>,
    ) -> Result<(), ConfigManagerError> {
        match self.load() {
            Ok(()) => Ok(()),
            Err(ConfigManagerError::ConfigFileNotFound) => {
                match fs::symlink_metadata(&self.config_path) {
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                        self.init(portable)?;
                        self.save()
                    }
                    _ => Err(ConfigManagerError::ConfigFileCorrupted),
                }
            }
            Err(error) => Err(error),
        }?;
        let discovered = pcl_core::minecraft::instance::folder::discover_repositories(
            &self.app_state.blocking_lock().repositories,
            portable,
            official,
        );
        if !discovered.is_empty() {
            self.update(|state| {
                state.repositories.extend(discovered);
                Ok(())
            })
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        }
        Ok(())
    }

    /// initialize the config file, for those who never use this launcher
    fn init(&self, portable: Option<&Path>) -> Result<(), ConfigManagerError> {
        let game_dir = super::paths::create_default_directory(&self.config_dir, portable)
            .map_err(|_| ConfigManagerError::ConfigDirNotFound)?;
        let mut state = self.app_state.blocking_lock();
        state.version_info = crate::config::metadata::default_version_info();
        state.active_repo_path = game_dir.clone();
        state.repositories.push(McFolder {
            name: "Default".into(),
            path: game_dir,
            kind: McFolderType::Original,
        });
        // CE 一致：首次运行不预置账号，由用户自行登录。
        Ok(())
    }

    /// Read known current fields independently; incompatible fields use defaults.
    fn load(&self) -> Result<(), ConfigManagerError> {
        let bytes = fs::read(&self.config_path).map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                ConfigManagerError::ConfigFileNotFound
            } else {
                ConfigManagerError::ConfigFileCorrupted
            }
        })?;
        let input = serde_yaml::from_slice::<serde_yaml::Value>(&bytes).unwrap_or_default();
        let mut rewrite =
            input.get("config_version").and_then(|v| v.as_u64()) != Some(CONFIG_VERSION);
        let mut fields = serde_yaml::Value::Mapping(Default::default());
        let mut state_read: AppState = serde_yaml::from_value(fields.clone())
            .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
        if let Some(object) = input.as_mapping() {
            for (key, value) in object {
                fields[key] = value.clone();
                match serde_yaml::from_value::<AppState>(fields.clone()) {
                    Ok(candidate) if valid_loaded_state(&candidate) => state_read = candidate,
                    _ => {
                        fields.as_mapping_mut().unwrap().remove(key);
                        rewrite = true;
                        log::warn!("Ignoring invalid configuration field: {key:?}");
                    }
                }
            }
        }
        #[cfg(windows)]
        normalize_loaded_paths(&mut state_read);
        if rewrite {
            let backup = self
                .config_path
                .with_extension(format!("{}.bak", uuid::Uuid::new_v4()));
            let mut options = fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            options.mode(0o600);
            options
                .open(&backup)
                .and_then(|mut file| file.write_all(&bytes))
                .map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
            save_config_atomically(&self.config_path, &state_read)?;
            log::info!(
                "Configuration backed up to {} and rewritten for {}",
                backup.display(),
                CONFIG_VERSION
            );
        }
        let (client, http1) = crate::config::network::clients(&state_read.proxy)
            .map_err(|_| ConfigManagerError::ApiClientBuildError)?;
        let mut state = self.app_state.blocking_lock();
        *state = state_read;
        self.api_client.replace_clients(client, http1);
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
        let path = pcl_core::utils::canonicalize(path).map_err(|error| error.to_string())?;
        self.update(|state| {
            if state.repositories.iter().any(|repo| {
                repo.name == name
                    || repo.path == path
                    || pcl_core::utils::canonicalize(&repo.path)
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
        settings: crate::config::download::DownloadSettings,
    ) -> Result<crate::config::download::DownloadSettings, String> {
        settings.validate()?;
        let mut state = self.app_state.blocking_lock();
        let mut candidate = state.clone();
        candidate.download_settings = Some(settings.clone());
        save_config_atomically(&self.config_path, &candidate).map_err(|e| format!("{e:?}"))?;
        *state = candidate;
        // 副作用放在提交之后：失败时磁盘与内存一致，用户重试即可恢复。
        self.api_client.resources().configure(settings.options())?;
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

    pub fn java_runtime_dir(&self) -> PathBuf {
        self.config_dir.join("runtime")
    }
}

#[cfg(windows)]
fn normalize_loaded_paths(state: &mut AppState) {
    use pcl_core::minecraft::instance::InstanceJava;

    fn path(value: &mut PathBuf) {
        *value = pcl_core::utils::simplify(value).to_owned();
    }
    fn java(entry: &mut pcl_core::minecraft::java::JavaEntry) {
        path(&mut entry.installation.java_exe);
        path(&mut entry.installation.java_folder);
    }
    for repo in &mut state.repositories {
        path(&mut repo.path);
    }
    path(&mut state.active_repo_path);
    if let Some(instance) = &mut state.active_game_instance {
        let instance = Arc::make_mut(instance);
        path(&mut instance.directory);
        path(&mut instance.jar_path);
        path(&mut instance.json_path);
        path(&mut instance.natives_path);
        path(&mut instance.global_dir);
        if let InstanceJava::Custom(entry) = &mut instance.game_java {
            java(Arc::make_mut(entry));
        }
    }
    for entry in &mut state.java_runtimes {
        java(entry);
    }
    if let Some(selected) = &mut state.launcher_setup.selected_java {
        path(selected);
    }
}

fn valid_loaded_state(state: &AppState) -> bool {
    !(state
        .log_max_lines
        .is_some_and(|lines| lines != 0 && !(50..=2000).contains(&lines))
        || state
            .window_settings
            .as_ref()
            .is_some_and(|settings| settings.validate().is_err())
        || state
            .memory_settings
            .as_ref()
            .is_some_and(|settings| settings.validate().is_err())
        || state
            .memory_custom
            .as_ref()
            .is_some_and(|custom| custom.validate().is_err())
        || state
            .download_settings
            .as_ref()
            .is_some_and(|settings| settings.validate().is_err())
        || state.link_settings.validate().is_err()
        || state.renderer > 3
        || state.proxy.validate().is_err())
}

fn save_config_atomically(
    path: &std::path::Path,
    value: &impl serde::Serialize,
) -> Result<(), ConfigManagerError> {
    let mut value =
        serde_yaml::to_value(value).map_err(|_| ConfigManagerError::ConfigFileCorrupted)?;
    value["config_version"] = CONFIG_VERSION.into();
    let temporary = path.with_extension(format!("{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        options.mode(0o600);
        let file = options
            .open(&temporary)
            .map_err(|_| ConfigManagerError::ConfigFileNotFound)?;
        let mut writer = std::io::BufWriter::new(file);
        serde_yaml::to_writer(&mut writer, &value)
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

#[cfg(test)]
mod java_config_tests {
    use super::*;
    use crate::config::states::WindowMode;
    use serde::ser::Error;
    #[test]
    fn window_settings_validate_and_preserve_failed_save() {
        let root =
            std::env::temp_dir().join(format!("pcl-window-settings-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let manager = ConfigManager {
            config_path: root.join("config.yml"),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(pcl_core::io::net::http::HttpTransport::default()),
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
            serde_yaml::from_slice(&fs::read(&manager.config_path).unwrap()).unwrap();
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
    fn initialization_recovers_config_and_creates_missing_config() {
        let root =
            std::env::temp_dir().join(format!("pcl-config-init-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let path = root.join("config.yml");
        let manager = ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(pcl_core::io::net::http::HttpTransport::default()),
            pcl_identifier: String::new(),
        };
        fs::write(&path, b"broken: [").unwrap();
        manager.load_or_init(None, None).unwrap();
        let backups = || {
            fs::read_dir(&root)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .filter(|path| path.extension().is_some_and(|ext| ext == "bak"))
                .collect::<Vec<_>>()
        };
        assert_eq!(backups().len(), 1);
        assert_eq!(fs::read(&backups()[0]).unwrap(), b"broken: [");
        let different_version = b"config_version: 0\nrenderer: 99\nlog_max_lines: 100\npcl_setup_info:\n  api_provider: bmclapi\n  max_memory: 4096\ndownload_settings:\n  file_source: 2\n  community_source: 0\n";
        fs::write(&path, different_version).unwrap();
        manager.load().unwrap();
        let state = manager.app_state.blocking_lock();
        assert!(state.java_runtimes.is_empty());
        assert_eq!(state.renderer, 0);
        assert_eq!(state.log_max_lines, Some(100));
        assert_eq!(state.launcher_setup.max_memory, 4096);
        assert!(matches!(
            state.download_settings.as_ref().unwrap().file_source,
            pcl_core::io::net::http::resources::SourcePreference::Official
        ));
        assert!(matches!(
            state.download_settings.as_ref().unwrap().community_source,
            pcl_core::io::net::http::resources::SourcePreference::Mirror
        ));
        drop(state);
        assert_eq!(backups().len(), 2);
        assert!(
            backups()
                .iter()
                .any(|path| fs::read(path).unwrap() == different_version)
        );
        let current = fs::read(&path).unwrap();
        assert_eq!(
            serde_yaml::from_slice::<serde_yaml::Value>(&current).unwrap()["config_version"],
            CONFIG_VERSION
        );
        manager.load().unwrap();
        assert_eq!(backups().len(), 2);
        assert_eq!(fs::read(&path).unwrap(), current);
        let damaged = String::from_utf8(current)
            .unwrap()
            .replace("renderer: 0", "renderer: 99");
        fs::write(&path, &damaged).unwrap();
        manager.load().unwrap();
        assert_eq!(backups().len(), 3);
        assert!(
            backups()
                .iter()
                .any(|path| fs::read(path).unwrap() == damaged.as_bytes())
        );
        assert_eq!(manager.app_state.blocking_lock().renderer, 0);
        fs::remove_file(&path).unwrap();
        let portable = root.join("portable/.minecraft");
        let official = root.join("official/.minecraft");
        fs::create_dir_all(official.join("versions")).unwrap();
        manager
            .load_or_init(Some(&portable), Some(&official))
            .unwrap();
        assert_eq!(manager.app_state.blocking_lock().version_info, "PCL.Proto");
        manager
            .update(|state| {
                state.version_info.clear();
                Ok(())
            })
            .unwrap();
        // Restart discovery must preserve selection and deduplicate canonical paths.
        manager.select_repository(1).unwrap();
        manager
            .load_or_init(Some(&portable.join(".")), Some(&official))
            .unwrap();
        let restored: AppState = serde_yaml::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert!(restored.version_info.is_empty());
        assert_eq!(restored.repositories.len(), 2);
        assert_eq!(
            restored.active_repo_path,
            pcl_core::utils::canonicalize(&official).unwrap()
        );
        assert!(portable.join("versions").is_dir());
        let blocked = root.join("blocked");
        fs::write(&blocked, b"file").unwrap();
        assert_eq!(
            crate::config::paths::create_default_directory(&root, Some(&blocked)).unwrap(),
            pcl_core::utils::canonicalize(root.join(".minecraft")).unwrap()
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn failed_saves_preserve_previous_config_and_successful_saves_replace_it() {
        let root =
            std::env::temp_dir().join(format!("pcl-java-config-test-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let path = root.join("config.yml");
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
        save_config_atomically(&path, &state).unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&path).unwrap().permissions().mode() & 0o777,
                0o600
            );
        }
        let restored: AppState = serde_yaml::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(
            restored.launcher_setup.selected_java,
            state.launcher_setup.selected_java
        );
        assert_eq!(fs::read_dir(&root).unwrap().count(), 1);
        fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn updates_publish_only_after_save_and_preserve_concurrent_changes() {
        let root = std::env::temp_dir().join(format!("pcl-config-update-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let path = root.join("config.yml");
        let manager = Arc::new(ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(pcl_core::io::net::http::HttpTransport::default()),
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
            .map(|_| {
                let manager = manager.clone();
                std::thread::spawn(move || {
                    manager
                        .update(|state| {
                            state.log_max_lines = Some(state.log_max_lines.unwrap_or(100) + 1);
                            Ok(())
                        })
                        .unwrap()
                })
            })
            .collect();
        for thread in threads {
            thread.join().unwrap();
        }
        let saved: AppState = serde_yaml::from_slice(&fs::read(&path).unwrap()).unwrap();
        assert_eq!(saved.log_max_lines, Some(108));
        assert_eq!(manager.app_state.blocking_lock().log_max_lines, Some(108));
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
        let path = root.join("config.yml");
        let manager = ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(pcl_core::io::net::http::HttpTransport::default()),
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
        let saved: AppState = serde_yaml::from_slice(&fs::read(&path).unwrap()).unwrap();
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
        use crate::config::download::DownloadSettings;
        let root =
            std::env::temp_dir().join(format!("pcl-download-settings-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).unwrap();
        let path = root.join("config.yml");
        let manager = ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(pcl_core::io::net::http::HttpTransport::default()),
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
        let restored: AppState = serde_yaml::from_slice(&saved).unwrap();
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
        assert_eq!(
            manager.api_client.resources().settings().max_connections,
            32
        );
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
            config_path: root.join("config.yml"),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState {
                repositories: vec![repo],
                active_repo_path: root.clone(),
                download_settings: Some(crate::config::download::DownloadSettings {
                    auto_select_instance: Some(false),
                    ..Default::default()
                }),
                ..Default::default()
            })),
            api_client: MinecraftApiClient::new(pcl_core::io::net::http::HttpTransport::default()),
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
            serde_yaml::from_slice(&fs::read(&manager.config_path).unwrap()).unwrap();
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
        let path = root.join("config.yml");
        let manager = ConfigManager {
            config_path: path.clone(),
            config_dir: root.clone(),
            app_state: Arc::new(Mutex::new(AppState::default())),
            api_client: MinecraftApiClient::new(pcl_core::io::net::http::HttpTransport::default()),
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
