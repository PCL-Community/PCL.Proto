//! PCL 实例配置边界。唯一持久化位置为 `PCL/config.v1.yml`。
//! 修改前重新读取并保留所有未知键；损坏的配置不会被默认值覆盖。
use super::{
    arguments::LaunchArguments,
    java_preference::{InstanceJavaSettings, JavaPreference},
    launch_settings::InstanceLaunchSettings,
    memory::{MemoryCustom, MemorySettings, MemorySnapshot},
};
use serde::{Serialize, de::DeserializeOwned};
use serde_yaml::{Mapping, Value};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::Mutex,
};

static WRITE_LOCK: Mutex<()> = Mutex::new(());

pub struct InstanceConfig {
    path: PathBuf,
    values: Mapping,
}

#[derive(Clone, Debug, Default, Serialize, serde::Deserialize)]
#[serde(default)]
pub struct InstanceMetadata {
    pub starred: bool,
    pub display_type: u8,
    pub logo: String,
    pub logo_custom: bool,
    pub description: String,
    pub launch_count: u64,
}

impl InstanceConfig {
    pub fn load(directory: &Path) -> Result<Self, String> {
        if !directory.is_absolute() || !directory.is_dir() {
            return Err(format!(
                "Instance directory is unavailable: {}",
                directory.display()
            ));
        }
        let parent = directory.join("PCL");
        let path = parent.join("config.v1.yml");
        for target in [directory, parent.as_path(), path.as_path()] {
            match fs::symlink_metadata(target) {
                Ok(metadata) if metadata.file_type().is_symlink() => {
                    return Err(format!(
                        "Instance configuration cannot be a symbolic link: {}",
                        target.display()
                    ));
                }
                Ok(_) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
        }
        let values = match fs::read(&path) {
            Ok(bytes) => match serde_yaml::from_slice::<Value>(&bytes)
                .map_err(|error| format!("{}: {error}", path.display()))?
            {
                Value::Mapping(values) => values,
                Value::Null
                    if String::from_utf8_lossy(&bytes).lines().all(|line| {
                        let line = line.trim().trim_start_matches('\u{feff}');
                        line.is_empty() || line.starts_with('#') || line == "---" || line == "..."
                    }) =>
                {
                    Mapping::new()
                }
                _ => {
                    return Err(format!(
                        "Instance configuration must be a YAML mapping: {}",
                        path.display()
                    ));
                }
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Mapping::new(),
            Err(error) => return Err(format!("{}: {error}", path.display())),
        };
        Ok(Self { path, values })
    }

    pub fn update<T>(
        directory: &Path,
        change: impl FnOnce(&mut Self) -> Result<T, String>,
    ) -> Result<T, String> {
        let _guard = WRITE_LOCK.lock().map_err(|error| error.to_string())?;
        let mut config = Self::load(directory)?;
        let original = config.values.clone();
        let result = change(&mut config)?;
        if config.values != original {
            config.save()?;
        }
        Ok(result)
    }

    fn save(&self) -> Result<(), String> {
        let parent = self
            .path
            .parent()
            .ok_or("Missing instance configuration directory")?;
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        let mut file =
            tempfile::NamedTempFile::new_in(parent).map_err(|error| error.to_string())?;
        serde_yaml::to_writer(&mut file, &self.values).map_err(|error| error.to_string())?;
        file.flush().map_err(|error| error.to_string())?;
        file.as_file()
            .sync_all()
            .map_err(|error| error.to_string())?;
        file.persist(&self.path)
            .map_err(|error| error.to_string())?;
        Ok(())
    }

    pub fn contains(&self, key: &str) -> bool {
        self.values.contains_key(Value::from(key))
    }

    pub fn get<T: DeserializeOwned>(&self, key: &str, default: T) -> Result<T, String> {
        let Some(value) = self.values.get(Value::from(key)) else {
            return Ok(default);
        };
        if let Ok(value) = serde_yaml::from_value(value.clone()) {
            return Ok(value);
        }
        if let Value::String(value) = value {
            if let Ok(value) = serde_yaml::from_str(value) {
                return Ok(value);
            }
            if let Ok(value) = serde_yaml::from_str(&value.to_ascii_lowercase()) {
                return Ok(value);
            }
        } else if matches!(value, Value::Number(_) | Value::Bool(_)) {
            if let Ok(value) = serde_yaml::from_value(Value::String(
                serde_yaml::to_string(value)
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_owned(),
            )) {
                return Ok(value);
            }
        }
        if matches!(value, Value::String(text) if text == "1" || text == "0")
            || matches!(value, Value::Number(number) if number.as_u64().is_some_and(|n| n <= 1))
        {
            let flag = value == &Value::String("1".into()) || value == &Value::from(1);
            if let Ok(value) = serde_yaml::from_value(Value::Bool(flag)) {
                return Ok(value);
            }
        }
        Err(format!(
            "Invalid instance setting {key} in {}",
            self.path.display()
        ))
    }

    pub fn set(&mut self, key: &str, value: impl Serialize) -> Result<(), String> {
        self.values.insert(
            Value::from(key),
            serde_yaml::to_value(value).map_err(|error| error.to_string())?,
        );
        Ok(())
    }

    pub fn arguments(&self) -> Result<LaunchArguments, String> {
        Ok(LaunchArguments {
            jvm: self.get("VersionAdvanceJvm", String::new())?,
            game: self.get("VersionAdvanceGame", String::new())?,
            classpath: self.get("VersionAdvanceClasspathHead", String::new())?,
        })
    }
    pub fn set_arguments(&mut self, settings: &LaunchArguments) -> Result<(), String> {
        settings.validate(true)?;
        self.set("VersionAdvanceJvm", &settings.jvm)?;
        self.set("VersionAdvanceGame", &settings.game)?;
        self.set("VersionAdvanceClasspathHead", &settings.classpath)
    }
    pub fn launch_settings(&self) -> Result<InstanceLaunchSettings, String> {
        Ok(InstanceLaunchSettings {
            info: self.get("VersionArgumentInfo", String::new())?,
            title: self.get("VersionArgumentTitle", String::new())?,
            use_global_title: self.get("VersionArgumentTitleEmpty", false)?,
            disable_mod_update: self.get("VersionAdvanceDisableModUpdate", false)?,
            server_login: self.get("VersionServerLoginRequire", 0)?,
            server_login_locked: self.get("VersionServerLoginLock", false)?,
            auth_server: self.get("VersionServerAuthServer", String::new())?,
            auth_register: self.get("VersionServerAuthRegister", String::new())?,
            auth_name: self.get("VersionServerAuthName", String::new())?,
            server_address: self.get("VersionServerEnter", String::new())?,
            renderer: self.get("VersionAdvanceRenderer", 0)?,
            prelaunch_command: self.get("VersionAdvanceRun", String::new())?,
            prelaunch_wait: self.get("VersionAdvanceRunWait", true)?,
            disable_asset_verify: self.get("VersionAdvanceAssetsV2", false)?,
            disable_jlw: self.get("VersionAdvanceDisableJLW", false)?,
            disable_legacy_fix: self.get("VersionAdvanceDisableLF", false)?,
            debug_log4j2: self.get("VersionUseDebugLog4j2Config", false)?,
            disable_lwjgl_unsafe_agent: self.get("VersionAdvanceDisableLwjglUnsafeAgent", false)?,
            use_proxy: self.get("VersionAdvanceUseProxyV2", false)?,
        })
    }
    pub fn set_launch_settings(&mut self, settings: &InstanceLaunchSettings) -> Result<(), String> {
        settings.validate()?;
        let old = self.launch_settings()?;
        if old.server_login_locked
            && (old.server_login != settings.server_login
                || old.auth_server != settings.auth_server
                || old.auth_register != settings.auth_register
                || old.auth_name != settings.auth_name
                || !settings.server_login_locked)
        {
            return Err("Instance login settings are locked".into());
        }
        macro_rules! fields { ($($key:literal => $field:ident),* $(,)?) => { $(self.set($key, &settings.$field)?;)* }; }
        fields! {
            "VersionArgumentInfo" => info, "VersionArgumentTitle" => title,
            "VersionArgumentTitleEmpty" => use_global_title, "VersionAdvanceDisableModUpdate" => disable_mod_update,
            "VersionServerLoginRequire" => server_login, "VersionServerLoginLock" => server_login_locked,
            "VersionServerAuthServer" => auth_server, "VersionServerAuthRegister" => auth_register,
            "VersionServerAuthName" => auth_name, "VersionServerEnter" => server_address,
            "VersionAdvanceRenderer" => renderer, "VersionAdvanceRun" => prelaunch_command,
            "VersionAdvanceRunWait" => prelaunch_wait, "VersionAdvanceAssetsV2" => disable_asset_verify,
            "VersionAdvanceDisableJLW" => disable_jlw, "VersionAdvanceDisableLF" => disable_legacy_fix,
            "VersionUseDebugLog4j2Config" => debug_log4j2,
            "VersionAdvanceDisableLwjglUnsafeAgent" => disable_lwjgl_unsafe_agent,
            "VersionAdvanceUseProxyV2" => use_proxy,
        }
        Ok(())
    }
    pub fn java(&self) -> Result<InstanceJavaSettings, String> {
        let value = self.get("VersionArgumentJavaSelect", "使用全局设置".to_owned())?;
        let preference = if value.trim().is_empty() {
            JavaPreference::Auto
        } else if value.trim() == "使用全局设置" {
            JavaPreference::Global
        } else if value.trim_start().starts_with('{') {
            #[derive(serde::Deserialize)]
            struct Preference {
                kind: String,
                javaexepath: Option<PathBuf>,
                relativepath: Option<PathBuf>,
            }
            let object: serde_json::Map<String, serde_json::Value> =
                serde_json::from_str(&value).map_err(|error| error.to_string())?;
            let normalized = object
                .into_iter()
                .map(|(key, value)| (key.to_ascii_lowercase(), value))
                .collect();
            let value: Preference = serde_json::from_value(serde_json::Value::Object(normalized))
                .map_err(|error| error.to_string())?;
            match value.kind.as_str() {
                "global" => JavaPreference::Global,
                "auto" => JavaPreference::Auto,
                "exist" => JavaPreference::Existing {
                    java_exe: value.javaexepath.ok_or("Missing JavaExePath")?,
                },
                "relative" => JavaPreference::Relative {
                    relative_path: value.relativepath.ok_or("Missing RelativePath")?,
                },
                _ => return Err("Invalid CE Java preference".into()),
            }
        } else {
            JavaPreference::Existing {
                java_exe: value.into(),
            }
        };
        Ok(InstanceJavaSettings {
            preference,
            ignore_compatibility: self.get("VersionAdvanceJava", false)?,
        })
    }
    pub fn set_java(&mut self, settings: &InstanceJavaSettings) -> Result<(), String> {
        let value = match &settings.preference {
            JavaPreference::Global => "使用全局设置".to_owned(),
            JavaPreference::Auto => serde_json::json!({"kind":"auto"}).to_string(),
            JavaPreference::Existing { java_exe } => {
                serde_json::json!({"kind":"exist", "JavaExePath":java_exe}).to_string()
            }
            JavaPreference::Relative { relative_path } => {
                serde_json::json!({"kind":"relative", "RelativePath":relative_path}).to_string()
            }
        };
        self.set("VersionArgumentJavaSelect", value)?;
        self.set("VersionAdvanceJava", settings.ignore_compatibility)
    }
    pub fn memory(&self) -> Result<MemorySnapshot, String> {
        let maximum = self.get::<u32>("VersionRamCustom", 15)?;
        let initial = self.get::<u32>("VersionRamCustomInitial", 0)?;
        let custom = MemoryCustom {
            maximum_mib: memory_tick_to_mib(maximum)?,
            initial_mib: if initial == 0 {
                None
            } else {
                Some(memory_tick_to_mib(initial)?)
            },
        };
        let settings = match self.get("VersionRamType", 2)? {
            0 => Some(MemorySettings::Auto),
            1 => Some(MemorySettings::Custom {
                maximum_mib: custom.maximum_mib,
                initial_mib: custom
                    .initial_mib
                    .map(|value| value.min(custom.maximum_mib)),
            }),
            2 => None,
            _ => return Err("Invalid VersionRamType".into()),
        };
        Ok(MemorySnapshot { settings, custom })
    }
    pub fn set_memory(
        &mut self,
        settings: Option<&MemorySettings>,
        custom: &MemoryCustom,
    ) -> Result<MemorySnapshot, String> {
        custom.validate()?;
        if let Some(settings) = settings {
            settings.validate()?;
        }
        self.set(
            "VersionRamType",
            match settings {
                None => 2,
                Some(MemorySettings::Auto) => 0,
                Some(MemorySettings::Custom { .. }) => 1,
            },
        )?;
        self.set("VersionRamCustom", memory_mib_to_tick(custom.maximum_mib))?;
        self.set(
            "VersionRamCustomInitial",
            custom
                .initial_mib
                .map(|value| memory_mib_to_tick(value).max(1))
                .unwrap_or(0),
        )?;
        self.memory()
    }
    pub fn metadata(&self) -> Result<InstanceMetadata, String> {
        Ok(InstanceMetadata {
            starred: self.get("IsStar", false)?,
            display_type: self.get("DisplayType", 0)?,
            logo: self.get("Logo", String::new())?,
            logo_custom: self.get("LogoCustom", false)?,
            description: self.get("CustomInfo", String::new())?,
            launch_count: self.get("VersionLaunchCount", 0)?,
        })
    }
}
fn memory_tick_to_mib(tick: u32) -> Result<u32, String> {
    let tick = f64::from(tick);
    let gib = if tick <= 12.0 {
        tick * 0.1 + 0.3
    } else if tick <= 25.0 {
        (tick - 12.0) * 0.5 + 1.5
    } else if tick <= 33.0 {
        tick - 25.0 + 8.0
    } else {
        (tick - 33.0) * 2.0 + 16.0
    };
    if gib * 1024.0 > u32::MAX as f64 {
        return Err("Invalid instance memory tick".into());
    }
    Ok((gib * 1024.0).floor() as u32)
}
fn memory_mib_to_tick(mib: u32) -> u32 {
    (0..=49)
        .min_by_key(|tick| memory_tick_to_mib(*tick).unwrap_or(u32::MAX).abs_diff(mib))
        .unwrap_or(15)
}

#[cfg(test)]
mod tests;
