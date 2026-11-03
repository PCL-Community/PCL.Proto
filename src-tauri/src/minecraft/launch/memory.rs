use crate::minecraft::{instance::McInstance, mod_loader::ModLoaderType};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum MemorySettings {
    Auto,
    Custom {
        maximum_mib: u32,
        initial_mib: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryCustom {
    pub maximum_mib: u32,
    pub initial_mib: Option<u32>,
}

impl Default for MemoryCustom {
    fn default() -> Self {
        Self {
            maximum_mib: 3072,
            initial_mib: None,
        }
    }
}

impl MemoryCustom {
    pub fn from_settings(settings: Option<&MemorySettings>) -> Self {
        match settings {
            Some(MemorySettings::Custom {
                maximum_mib,
                initial_mib,
            }) => Self {
                maximum_mib: *maximum_mib,
                initial_mib: *initial_mib,
            },
            _ => Self::default(),
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if !(307..=49152).contains(&self.maximum_mib)
            || self
                .initial_mib
                .is_some_and(|value| !(307..=49152).contains(&value))
        {
            return Err("Invalid custom game memory allocation".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySnapshot {
    pub settings: Option<MemorySettings>,
    pub custom: MemoryCustom,
}

impl MemorySettings {
    pub fn validate(&self) -> Result<(), String> {
        let Self::Custom {
            maximum_mib,
            initial_mib,
        } = self
        else {
            return Ok(());
        };
        if !(307..=49152).contains(maximum_mib)
            || initial_mib.is_some_and(|value| value < 307 || value > *maximum_mib)
        {
            return Err("Invalid game memory allocation".into());
        }
        Ok(())
    }
}

pub fn automatic(
    available_bytes: u64,
    modded: bool,
    optifine: bool,
    mod_count: usize,
) -> Result<usize, String> {
    if available_bytes == 0 {
        return Err("Available physical memory is unavailable".into());
    }
    let mods = mod_count as f64;
    let (min, t1, t2, t3) = if modded {
        (
            0.5 + mods / 150.0,
            1.5 + mods / 90.0,
            2.7 + mods / 50.0,
            4.5 + mods / 25.0,
        )
    } else if optifine {
        (0.5, 1.5, 3.0, 5.0)
    } else {
        (0.5, 1.5, 2.5, 4.0)
    };
    let mut available = (available_bytes as f64 / 1_073_741_824.0 * 10.0).round() / 10.0;
    let mut given: f64 = 0.0;
    for (delta, ratio) in [(t1, 1.0), (t2 - t1, 0.7), (t3 - t2, 0.4), (t3, 0.15)] {
        given += (available * ratio).min(delta);
        available -= delta / ratio;
        if available < 0.1 {
            break;
        }
    }
    Ok(((given.max(min) * 10.0).round() / 10.0 * 1024.0).floor() as usize)
}

pub fn resolve(
    settings: &MemorySettings,
    available_bytes: u64,
    instance: &McInstance,
    game_directory: &std::path::Path,
    is_64_bit: bool,
) -> Result<(usize, Option<usize>), String> {
    settings.validate()?;
    let (maximum, initial) = match settings {
        MemorySettings::Custom {
            maximum_mib,
            initial_mib,
        } => (
            *maximum_mib as usize,
            initial_mib.map(|value| value as usize),
        ),
        MemorySettings::Auto => {
            let modded = matches!(
                instance.plugin_type,
                ModLoaderType::Forge
                    | ModLoaderType::NeoForge
                    | ModLoaderType::Fabric
                    | ModLoaderType::LegacyFabric
                    | ModLoaderType::Quilt
            );
            let mut mod_count = 0;
            if modded {
                match std::fs::read_dir(game_directory.join("mods")) {
                    Ok(entries) => {
                        for entry in entries {
                            let entry = entry.map_err(|error| error.to_string())?;
                            if entry
                                .file_type()
                                .map_err(|error| error.to_string())?
                                .is_file()
                            {
                                mod_count += 1;
                            }
                        }
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                    Err(error) => {
                        return Err(format!(
                            "Cannot inspect mods for memory allocation: {error}"
                        ));
                    }
                }
            }
            (
                automatic(
                    available_bytes,
                    modded,
                    matches!(instance.plugin_type, ModLoaderType::Optifine),
                    mod_count,
                )?,
                None,
            )
        }
    };
    let maximum = if is_64_bit {
        maximum
    } else {
        maximum.min(1024)
    };
    Ok((maximum, initial.map(|value| value.min(maximum))))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn system_memory_is_available_for_auto_allocation() {
        let mut system = sysinfo::System::new().unwrap();
        system.refresh_memory();
        assert!(system.total_memory() > 0);
        let available = system.available_memory().min(system.total_memory());
        assert!(
            available > 0,
            "System reported no available physical memory"
        );
        assert!(automatic(available, true, false, 0).is_ok());
    }
    #[test]
    fn ce_automatic_allocation_stages_and_validation() {
        const GIB: u64 = 1_073_741_824;
        assert_eq!(automatic(GIB, false, false, 0).unwrap(), 1024);
        assert_eq!(automatic(4 * GIB, false, false, 0).unwrap(), 2969);
        assert!(
            automatic(8 * GIB, true, false, 100).unwrap()
                > automatic(8 * GIB, false, false, 0).unwrap()
        );
        assert!(automatic(0, false, false, 0).is_err());
        assert!(
            MemorySettings::Custom {
                maximum_mib: 1024,
                initial_mib: Some(2048)
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn automatic_counts_mods_in_the_resolved_game_directory() {
        use crate::minecraft::instance::InstanceJava;
        let root = std::env::temp_dir().join(format!("pcl-memory-dir-{}", uuid::Uuid::new_v4()));
        let instance_dir = root.join("version");
        let shared_dir = root.join("shared");
        std::fs::create_dir_all(instance_dir.join("mods")).unwrap();
        std::fs::create_dir_all(shared_dir.join("mods")).unwrap();
        for index in 0..100 {
            std::fs::write(shared_dir.join("mods").join(format!("{index}.jar")), b"").unwrap();
        }
        let instance = McInstance {
            id: "test".into(),
            name: "test".into(),
            directory: instance_dir.clone(),
            jar_path: instance_dir.join("test.jar"),
            version: "1.0".into(),
            json_path: instance_dir.join("test.json"),
            natives_path: instance_dir.join("natives"),
            game_java: InstanceJava::Default,
            global_dir: shared_dir.clone(),
            plugin_type: ModLoaderType::Forge,
            plugins_version: Default::default(),
        };
        let available = 8 * 1_073_741_824;
        let isolated = resolve(
            &MemorySettings::Auto,
            available,
            &instance,
            &instance_dir,
            true,
        )
        .unwrap()
        .0;
        let shared = resolve(
            &MemorySettings::Auto,
            available,
            &instance,
            &shared_dir,
            true,
        )
        .unwrap()
        .0;
        assert_eq!(isolated, automatic(available, true, false, 0).unwrap());
        assert_eq!(shared, automatic(available, true, false, 100).unwrap());
        assert!(shared > isolated);
        std::fs::remove_dir_all(root).unwrap();
    }
}
