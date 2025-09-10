use std::{path::PathBuf, sync::Arc};

use crate::core::java::JavaRuntime;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum GameJava {
    Default,
    Custom(Arc<JavaRuntime>),
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
}

impl GameInstance {
    // for test
    pub fn new(name: String, directory: PathBuf, version: String) -> Self {
        Self {
            id: name.clone(),
            name: name.clone(),
            directory: directory.clone(),
            version,
            jar_path: directory.join(format!("{}.jar", name)),
            json_path: directory.join(format!("{}.json", name)),
            natives_path: directory.join("natives"),
            game_java: GameJava::Default,
        }
    }
}
