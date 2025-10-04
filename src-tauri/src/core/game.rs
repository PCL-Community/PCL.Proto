use std::{fs, path::PathBuf, sync::Arc};

use crate::{core::java::JavaRuntime, core::repository::GameRepository};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GameJava {
    Default,
    Custom(Arc<JavaRuntime>),
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum PluginType {
    Vanilla,
    Forge,
    NeoForge,
    Fabric,
    Quilt,
    Optifine,
}

impl Default for PluginType {
    fn default() -> Self {
        PluginType::Vanilla
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GameInstance {
    pub id: String,
    pub name: String,
    pub directory: PathBuf,
    pub jar_path: PathBuf,
    pub version: String,
    pub json_path: PathBuf,
    pub natives_path: PathBuf,
    pub game_java: GameJava,
    pub global_dir: GameRepository,
    pub plugin_type: PluginType,
}

#[derive(Debug)]
pub enum GameInstanceError {
    InvalidVersionFolder,
    InvalidVersionJson,
    InvalidVersionJar,
}

impl GameInstance {
    pub fn from_version_folder(
        version_folder: &PathBuf,
        repo: &GameRepository,
    ) -> Result<Self, GameInstanceError> {
        let folder_name = version_folder
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(GameInstanceError::InvalidVersionFolder)?;
        let entries: fs::ReadDir =
            fs::read_dir(version_folder).map_err(|_| GameInstanceError::InvalidVersionFolder)?;
        let mut json_files = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().map_or(false, |ext| ext == "json"));

        if let Some(json_path) = crate::util::file::find_file_of_name(&mut json_files, folder_name)
        {
            let json_reader =
                fs::File::open(&json_path).map_err(|_| GameInstanceError::InvalidVersionJson)?;
            let json_content: serde_json::Value = serde_json::from_reader(json_reader)
                .map_err(|_| GameInstanceError::InvalidVersionJson)?;
            let id = json_content["id"]
                .as_str()
                .ok_or(GameInstanceError::InvalidVersionJson)?;
            let jar_name = if let Some(jar_name_in_json) = json_content["jar"].as_str() {
                jar_name_in_json // use jar name recorded in json file if there is
            } else {
                folder_name // use folder name as jar name if there is no jar name in json file
            };
            let jar_path = version_folder.join(format!("{}.jar", jar_name));
            if !jar_path.exists() {
                log::error!("jar file not found in folder: {:?}", jar_path);
                return Err(GameInstanceError::InvalidVersionJar);
            }
            let version = {
                let patches = json_content["patches"].as_array();
                if let Some(patches) = patches {
                    let game_patch = patches.iter().find(|patch| patch["id"] == "game");
                    if let Some(game_patch) = game_patch {
                        game_patch["version"].as_str()
                    } else {
                        None
                    }
                } else {
                    None
                }
            };
            let version = version.unwrap_or(id).to_string();
            return Ok(GameInstance {
                id: id.to_string(),
                name: folder_name.to_string(),
                directory: version_folder.clone(),
                jar_path,
                version,
                json_path,
                natives_path: version_folder.join("natives"),
                game_java: GameJava::Default,
                global_dir: repo.clone(),
                plugin_type: PluginType::default(),
            });
        } else {
            log::error!("version json not found in folder: {:?}", version_folder);
            return Err(GameInstanceError::InvalidVersionJson);
        }
    }
}

#[cfg(test)]
#[test]
fn from_version_folder() {
    let version_folder =
        PathBuf::from("/Users/amagicpear/HMCL/.minecraft/versions/Fabulously Optimized 1.21.5");
    let game_repo = GameRepository::new("HMCL", PathBuf::from("/Users/amagicpear/HMCL/.minecraft"));
    let game_repo = Arc::new(game_repo);
    let instance = GameInstance::from_version_folder(&version_folder, &game_repo);
    println!("{:?}", instance);
}
