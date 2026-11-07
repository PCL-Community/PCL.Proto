//! 把配置折算成一次启动的输入；启动本身由 `pcl-core` 执行。

use std::sync::Arc;

use crate::config::arguments::LaunchArguments;
use pcl_core::minecraft::java::JavaEntry;
use pcl_core::minecraft::launch::launcher::{
    GameLaunchError, LaunchOption, LaunchRequest, LauncherBrand,
};

use crate::config::manager::ConfigManager;
use crate::config::metadata::constants::{APP_VERSION, LAUNCHER_NAME};
use crate::config::states::{AppState, WindowMode};

pub fn request(state: &AppState, java: Arc<JavaEntry>) -> Result<LaunchOption, GameLaunchError> {
    let Some(game_instance) = state.active_game_instance.as_ref() else {
        return Err(GameLaunchError::MissingInstance);
    };
    let Some(account) = state.profile.current() else {
        return Err(GameLaunchError::MissingAccount);
    };
    let game_directory =
        crate::config::isolation::directory(&state.instance_isolation, game_instance).to_owned();
    let (max_memory, initial_memory) = match state
        .instance_memory_settings
        .get(&game_instance.directory)
        .or(state.memory_settings.as_ref())
    {
        Some(settings) => {
            let available = if matches!(settings, crate::config::memory::MemorySettings::Auto) {
                match sysinfo::System::new() {
                    Ok(mut system) => {
                        system.refresh_memory();
                        system.available_memory()
                    }
                    Err(_) => 0,
                }
            } else {
                0
            };
            crate::config::memory::resolve(
                settings,
                available,
                game_instance,
                &game_directory,
                java.is_64_bit,
            )
            .map_err(GameLaunchError::Memory)?
        }
        None => (state.launcher_setup.max_memory, None),
    };
    let global = state
        .launch_arguments
        .clone()
        .unwrap_or_else(LaunchArguments::global_default);
    let arguments = state
        .instance_launch_arguments
        .get(&game_instance.directory)
        .map(|instance| instance.effective(&global))
        .unwrap_or_else(|| global.effective(&global));
    let window = state.window_settings.as_ref();
    let custom = window.filter(|settings| settings.mode == WindowMode::Custom);
    let config = ConfigManager::instance();
    LaunchOption::new(LaunchRequest {
        wrapper_temp_dir: config.config_dir().to_path_buf(),
        helper_directory: config.config_dir().join("resources"),
        brand: LauncherBrand {
            name: LAUNCHER_NAME.into(),
            version: APP_VERSION.into(),
        },
        helpers: crate::assets::Assets::new(config.config_dir()).launch_helpers()?,
        instance: Arc::clone(game_instance),
        profile: account.clone(),
        java,
        game_directory,
        max_memory,
        initial_memory,
        width: custom.map(|settings| settings.width as usize),
        height: custom.map(|settings| settings.height as usize),
        arguments,
        extras: crate::config::launch_settings::resolve_for(state, Some(&game_instance.directory)),
    })
}
