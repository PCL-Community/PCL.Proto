//! 实例启动附加设置：读写配置并折算成 `pcl-core` 的启动参数。

use std::path::Path;

use crate::config::java_preference::current_instance;
use pcl_core::minecraft::launch::settings::LaunchExtras;

use crate::config::states::AppState;

pub fn update(
    state: &mut AppState,
    instance_directory: &Path,
    settings: InstanceLaunchSettings,
) -> Result<InstanceLaunchSettings, String> {
    settings.validate()?;
    current_instance(state.active_game_instance.as_deref(), instance_directory)
        .map_err(|error| error.to_string())?;
    if settings == InstanceLaunchSettings::default() {
        state.instance_launch_settings.remove(instance_directory);
    } else {
        state
            .instance_launch_settings
            .insert(instance_directory.to_owned(), settings.clone());
    }
    Ok(settings)
}

/// 把全局开关和实例覆盖折算成实际生效的启动选项。
pub fn resolve_for(state: &AppState, instance: Option<&Path>) -> LaunchExtras {
    let instance_settings = instance.and_then(|path| state.instance_launch_settings.get(path));
    resolve(state, instance_settings)
}

/// 0 follows the global renderer, 1..=4 map to the global numbering 0..=3.
pub const RENDERER_FOLLOW_GLOBAL: u8 = 0;
pub const RENDERER_MAX: u8 = 4;

#[derive(Clone, Debug, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct InstanceLaunchSettings {
    /// Custom version info shown in the game; empty follows the global value.
    pub info: String,
    pub renderer: u8,
    pub prelaunch_command: String,
    pub prelaunch_wait: bool,
    pub disable_asset_verify: bool,
    pub disable_jlw: bool,
    pub disable_legacy_fix: bool,
    pub debug_log4j2: bool,
    pub disable_lwjgl_unsafe_agent: bool,
    pub use_proxy: bool,
}

impl InstanceLaunchSettings {
    pub fn validate(&self) -> Result<(), String> {
        if self.renderer > RENDERER_MAX {
            return Err("Invalid renderer choice".into());
        }
        validate_info(&self.info)?;
        if self.prelaunch_command.len() > 16384 || self.prelaunch_command.contains('\0') {
            return Err("Invalid instance launch setting".into());
        }
        Ok(())
    }
}

pub fn resolve(
    global: &AppState,
    instance_settings: Option<&InstanceLaunchSettings>,
) -> LaunchExtras {
    let settings = instance_settings.cloned().unwrap_or_default();
    // Empty info stays empty; the launcher drops `--versionType` entirely, as CE does.
    let info = if settings.info.trim().is_empty() {
        global.version_info.clone()
    } else {
        settings.info.clone()
    };
    let renderer = if settings.renderer == RENDERER_FOLLOW_GLOBAL {
        global.renderer
    } else {
        settings.renderer - 1
    }
    .min(3);
    let proxy = settings
        .use_proxy
        .then_some(&global.proxy)
        .and_then(|settings| settings.game_endpoint());
    LaunchExtras {
        info,
        renderer,
        prelaunch_command: settings.prelaunch_command,
        prelaunch_wait: settings.prelaunch_wait,
        disable_asset_verify: settings.disable_asset_verify,
        java_wrapper: !global.disable_jlw && !settings.disable_jlw,
        legacy_fix: !global.disable_legacy_fix && !settings.disable_legacy_fix,
        debug_log4j2: settings.debug_log4j2,
        lwjgl_unsafe_agent: !global.disable_lwjgl_unsafe_agent
            && !settings.disable_lwjgl_unsafe_agent,
        proxy,
    }
}

/// CE's `LaunchArgumentInfo` rule: bounded length, no NUL, and no quotes, which would
/// otherwise leak into the `--versionType` argument.
pub fn validate_info(value: &str) -> Result<(), String> {
    if value.len() > 16384
        || value
            .chars()
            .any(|c| matches!(c, '\0' | '"' | '\u{201c}' | '\u{201d}'))
    {
        return Err("Invalid version info".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn global() -> AppState {
        AppState {
            version_info: "PCL.Proto".into(),
            disable_jlw: false,
            ..Default::default()
        }
    }

    #[test]
    fn resolves_global_defaults_and_instance_overrides() {
        let mut global = global();
        let mut settings = InstanceLaunchSettings {
            renderer: RENDERER_FOLLOW_GLOBAL,
            use_proxy: true,
            disable_jlw: true,
            ..Default::default()
        };
        let extras = resolve(&global, Some(&settings));
        assert_eq!(extras.info, "PCL.Proto");
        assert_eq!(extras.renderer, 0);
        assert!(!extras.java_wrapper);
        assert_eq!(extras.proxy, None);
        assert!(extras.legacy_fix);
        assert!(extras.lwjgl_unsafe_agent);

        global.renderer = 2;
        let extras = resolve(&global, Some(&settings));
        assert_eq!(extras.renderer, 2);
        settings.renderer = 4;
        let extras = resolve(&global, Some(&settings));
        assert_eq!(extras.renderer, 3);
        assert_eq!(resolve(&global, None).info, "PCL.Proto");
        global.version_info = String::new();
        assert!(resolve(&global, None).info.is_empty());
    }

    #[test]
    fn validates_renderer_and_command_bounds() {
        assert!(
            InstanceLaunchSettings {
                renderer: RENDERER_MAX + 1,
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(
            InstanceLaunchSettings {
                prelaunch_command: "a\0b".into(),
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(InstanceLaunchSettings::default().validate().is_ok());
        assert!(
            InstanceLaunchSettings {
                info: "PCL \"Proto\"".into(),
                ..Default::default()
            }
            .validate()
            .is_err()
        );
        assert!(validate_info("PCL.Proto").is_ok());
    }
}
