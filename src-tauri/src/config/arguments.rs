//! 启动参数设置、默认值与全局/实例继承。

use std::path::Path;

use crate::config::java_preference::current_instance;

use crate::config::states::AppState;

pub fn update(
    state: &mut AppState,
    instance_directory: Option<&Path>,
    settings: LaunchArguments,
) -> Result<LaunchArguments, String> {
    settings.validate(instance_directory.is_some())?;
    if let Some(path) = instance_directory {
        current_instance(state.active_game_instance.as_deref(), path)
            .map_err(|error| error.to_string())?;
        state
            .instance_launch_arguments
            .insert(path.to_owned(), settings.clone());
    } else {
        state.launch_arguments = Some(settings.clone());
    }
    Ok(settings)
}

pub const DEFAULT_JVM: &str = "-XX:+UseG1GC -XX:-UseAdaptiveSizePolicy -XX:-OmitStackTraceInFastThrow -Djdk.lang.Process.allowAmbiguousCommands=true -Dfml.ignoreInvalidMinecraftCertificates=True -Dfml.ignorePatchDiscrepancies=True -Dlog4j2.formatMsgNoLookups=true";

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct LaunchArguments {
    pub jvm: String,
    pub game: String,
    pub classpath: String,
}

impl LaunchArguments {
    pub fn global_default() -> Self {
        Self {
            jvm: DEFAULT_JVM.into(),
            ..Self::default()
        }
    }

    pub fn validate(&self, instance: bool) -> Result<(), String> {
        if !instance && !self.classpath.is_empty() {
            return Err("Classpath can only be set for an instance".into());
        }
        self.effective(self).validate()
    }

    pub fn effective(
        &self,
        global: &Self,
    ) -> pcl_core::minecraft::launch::arguments::LaunchArguments {
        pcl_core::minecraft::launch::arguments::LaunchArguments {
            jvm: if self.jvm.is_empty() {
                global.jvm.clone()
            } else {
                self.jvm.clone()
            },
            game: if self.game.is_empty() {
                global.game.clone()
            } else {
                self.game.clone()
            },
            classpath: self.classpath.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_arguments_and_inherits_each_field_independently() {
        let global = LaunchArguments {
            jvm: "-Dglobal=true".into(),
            game: "--demo".into(),
            classpath: String::new(),
        };
        let instance = LaunchArguments {
            jvm: String::new(),
            game: "--name 'with space'".into(),
            classpath: std::env::temp_dir()
                .join("extra library.jar")
                .to_string_lossy()
                .into_owned(),
        };
        instance.validate(true).unwrap();
        let effective = instance.effective(&global);
        assert_eq!(effective.jvm, global.jvm);
        assert_eq!(effective.game, instance.game);
        assert_eq!(effective.classpath_paths().unwrap().len(), 1);
        assert!(instance.validate(false).is_err());
        assert!(
            LaunchArguments {
                jvm: "'unclosed".into(),
                ..Default::default()
            }
            .validate(true)
            .is_err()
        );
        assert!(
            LaunchArguments {
                game: "bad\0value".into(),
                ..Default::default()
            }
            .validate(true)
            .is_err()
        );
        assert!(
            LaunchArguments {
                classpath: "relative.jar".into(),
                ..Default::default()
            }
            .validate(true)
            .is_err()
        );
    }
}
