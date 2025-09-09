use std::path::PathBuf;

pub struct GameInstance {
    id: String,
    name: String,
    pub directory: PathBuf,
    pub jar_path: PathBuf,
    pub version: String,
    pub json_path: PathBuf,
    pub natives_path: PathBuf,
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
        }
    }
}
