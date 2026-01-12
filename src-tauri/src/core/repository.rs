use std::{fs, path::PathBuf, sync::OnceLock};

use crate::core::game::GameInstance;

/// a .minecraft folder, not a single version folder
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GameRepository {
    pub name: String,
    pub path: PathBuf,
    #[serde(skip)]
    #[serde(default = "OnceLock::new")]
    game_instances: OnceLock<Vec<GameInstance>>,
}

impl GameRepository {
    pub fn new(name: &str, path: PathBuf) -> Self {
        Self {
            name: name.to_string(),
            path,
            game_instances: OnceLock::new(),
        }
    }
    pub fn game_instances(&self) -> &[GameInstance] {
        self.game_instances.get_or_init(|| {
            let versions_folder = self.path.join("versions");
            let mut game_instances = Vec::new();
            let entries = fs::read_dir(versions_folder);
            if let Err(e) = entries {
                log::warn!("failed to read versions folder: {:?} at {:?}", e, self.path);
                return game_instances;
            }
            for entry in entries.unwrap() {
                let entry = entry.unwrap();
                let path = entry.path();
                if path.is_dir() {
                    let game_instance_result = GameInstance::from_version_folder(&path, self);
                    if let Ok(game_instance) = game_instance_result {
                        log::debug!("loaded game instance: {:?}", &game_instance.id);
                        game_instances.push(game_instance);
                    } else {
                        log::error!(
                            "failed to load game instance with error: {:?} at {:?}",
                            game_instance_result.unwrap_err(),
                            path
                        );
                    }
                }
            }
            game_instances
        })
    }
}
