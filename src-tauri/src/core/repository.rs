use std::{fs, path::PathBuf};

use crate::core::game::GameInstance;

/// a .minecraft folder, not a single version folder
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct GameRepository {
    pub name: String,
    pub path: PathBuf,
}

impl GameRepository {
    pub fn game_instances(&self) -> Vec<GameInstance> {
        let versions_folder = self.path.join("versions");
        let mut game_instances = Vec::new();
        let entries = fs::read_dir(versions_folder);
        if entries.is_err() {
            log::warn!("failed to read versions folder: {:?}", entries.unwrap_err());
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
    }
}

#[test]
fn test_game_instances() {
    let game_repository = GameRepository {
        name: "test".to_string(),
        path: PathBuf::from("/Users/amagicpear/HMCL/.minecraft"),
    };
    let game_instances = game_repository.game_instances();
    println!("{:?}", game_instances.len());
}
