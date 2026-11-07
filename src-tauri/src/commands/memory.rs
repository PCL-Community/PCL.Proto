use crate::config::{current_instance, manager::ConfigManager};

#[tauri::command]
pub async fn get_memory_settings(
    instance_directory: Option<std::path::PathBuf>,
) -> Result<crate::config::memory::MemorySnapshot, String> {
    use crate::config::memory::{MemoryCustom, MemorySettings, MemorySnapshot};
    let state = ConfigManager::instance().app_state.lock().await;
    match instance_directory {
        Some(directory) => {
            current_instance(&state, &directory).map_err(|error| error.to_string())?;
            let settings = state.instance_memory_settings.get(&directory).cloned();
            let custom = state
                .instance_memory_custom
                .get(&directory)
                .cloned()
                .unwrap_or_else(|| MemoryCustom::from_settings(settings.as_ref()));
            Ok(MemorySnapshot { settings, custom })
        }
        None => {
            let settings = Some(
                state
                    .memory_settings
                    .clone()
                    .unwrap_or(MemorySettings::Custom {
                        maximum_mib: u32::try_from(state.launcher_setup.max_memory)
                            .map_err(|error| error.to_string())?,
                        initial_mib: None,
                    }),
            );
            let custom = state
                .memory_custom
                .clone()
                .unwrap_or_else(|| MemoryCustom::from_settings(settings.as_ref()));
            Ok(MemorySnapshot { settings, custom })
        }
    }
}

#[tauri::command]
pub async fn set_memory_settings(
    instance_directory: Option<std::path::PathBuf>,
    settings: Option<crate::config::memory::MemorySettings>,
    custom: Option<crate::config::memory::MemoryCustom>,
) -> Result<crate::config::memory::MemorySnapshot, String> {
    use crate::config::memory::{MemoryCustom, MemorySnapshot};
    if let Some(settings) = &settings {
        settings.validate()?;
    }
    if let Some(custom) = &custom {
        custom.validate()?;
    }
    if let (
        Some(crate::config::memory::MemorySettings::Custom {
            maximum_mib,
            initial_mib,
        }),
        Some(custom),
    ) = (&settings, &custom)
    {
        let mismatch = *maximum_mib != custom.maximum_mib
            || *initial_mib
                != custom
                    .initial_mib
                    .map(|value| value.min(custom.maximum_mib));
        if mismatch {
            return Err("Custom memory values do not match the active settings".into());
        }
    }
    tauri::async_runtime::spawn_blocking(move || {
        ConfigManager::instance().update(|state| {
            if let Some(directory) = instance_directory {
                current_instance(state, &directory).map_err(|error| error.to_string())?;
                if let Some(settings) = settings.clone() {
                    state
                        .instance_memory_settings
                        .insert(directory.clone(), settings);
                } else {
                    state.instance_memory_settings.remove(&directory);
                }
                let custom =
                    custom.unwrap_or_else(|| MemoryCustom::from_settings(settings.as_ref()));
                state
                    .instance_memory_custom
                    .insert(directory.clone(), custom.clone());
                Ok(MemorySnapshot { settings, custom })
            } else {
                let value = settings
                    .clone()
                    .ok_or("Global memory settings cannot inherit")?;
                state.memory_settings = Some(value.clone());
                let custom = custom.unwrap_or_else(|| MemoryCustom::from_settings(Some(&value)));
                state.memory_custom = Some(custom.clone());
                Ok(MemorySnapshot {
                    settings: Some(value),
                    custom,
                })
            }
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[derive(serde::Serialize)]
pub struct MemoryPreview {
    total_bytes: u64,
    available_bytes: u64,
    allocation_mib: Option<usize>,
}

#[tauri::command]
pub async fn preview_memory_settings(
    instance_directory: Option<std::path::PathBuf>,
    settings: Option<crate::config::memory::MemorySettings>,
    is_64_bit: Option<bool>,
) -> Result<MemoryPreview, String> {
    tauri::async_runtime::spawn_blocking(move || {
        use crate::config::memory::{self, MemorySettings};
        let config = ConfigManager::instance();
        let path = {
            let state = config.app_state.blocking_lock();
            instance_directory.clone().or_else(|| {
                state
                    .active_game_instance
                    .as_ref()
                    .map(|instance| instance.directory.clone())
            })
        };
        if let Some(path) = &path {
            config.ensure_instance_isolation(path)?;
        }
        let (instance, effective) = {
            let state = config.app_state.blocking_lock();
            let instance = if let Some(path) = &instance_directory {
                Some(
                    current_instance(&state, path)
                        .map_err(|error| error.to_string())?
                        .clone(),
                )
            } else {
                state.active_game_instance.as_deref().cloned()
            };
            let effective = settings
                .or_else(|| state.memory_settings.clone())
                .unwrap_or(MemorySettings::Custom {
                    maximum_mib: u32::try_from(state.launcher_setup.max_memory)
                        .map_err(|error| error.to_string())?,
                    initial_mib: None,
                });
            let game_directory = instance.as_ref().map(|instance| {
                crate::config::isolation::directory(&state.instance_isolation, instance)
                    .to_path_buf()
            });
            (
                instance.map(|instance| (instance, game_directory.unwrap())),
                effective,
            )
        };
        effective.validate()?;
        let mut system = sysinfo::System::new().map_err(|_| "Physical memory is unavailable")?;
        system.refresh_memory();
        let total = system.total_memory();
        if total == 0 {
            return Err("Physical memory is unavailable".into());
        }
        let available = system.available_memory().min(total);
        let allocation = match is_64_bit {
            None => None,
            Some(is_64_bit) => {
                let value = if let Some((instance, game_directory)) = &instance {
                    memory::resolve(&effective, available, instance, game_directory, is_64_bit)?.0
                } else {
                    let max = match effective {
                        MemorySettings::Auto => memory::automatic(available, false, false, 0)?,
                        MemorySettings::Custom { maximum_mib, .. } => maximum_mib as usize,
                    };
                    if is_64_bit { max } else { max.min(1024) }
                };
                Some(value)
            }
        };
        Ok(MemoryPreview {
            total_bytes: total,
            available_bytes: available,
            allocation_mib: allocation,
        })
    })
    .await
    .map_err(|error| error.to_string())?
}
