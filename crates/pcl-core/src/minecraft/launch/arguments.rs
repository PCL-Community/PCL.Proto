use std::path::PathBuf;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LaunchArguments {
    pub jvm: String,
    pub game: String,
    pub classpath: String,
}

impl LaunchArguments {
    pub fn validate(&self) -> Result<(), String> {
        for (name, value) in [("JVM", &self.jvm), ("Game", &self.game)] {
            if value.len() > 16384 || value.contains('\0') || shlex::split(value).is_none() {
                return Err(format!(
                    "Invalid {name} arguments: check quotes and escapes"
                ));
            }
        }
        self.classpath_paths()?;
        Ok(())
    }

    pub fn classpath_paths(&self) -> Result<Vec<PathBuf>, String> {
        if self.classpath.len() > 16384 || self.classpath.contains('\0') {
            return Err("Invalid classpath".into());
        }
        self.classpath
            .split(';')
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| {
                let path = PathBuf::from(value);
                if !path.is_absolute() {
                    return Err(format!("Classpath path must be absolute: {value}"));
                }
                Ok(path)
            })
            .collect()
    }
}
