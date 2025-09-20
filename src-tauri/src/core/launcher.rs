//! Mod for launching a Minecraft game
//! Code referenced from xphost/MMCLL/src/launcher.rs
//! MIT License | https://github.com/xphost008/MMCLL
//! Code referenced from scl-core/src/client.rs
//! LGPL-3.0 License & scl License | https://github.com/Steve-xmh/scl
//! Start script referenced from HMCL
//! GPL-3.0 License | https://github.com/HMCL-dev/HMCL
// use crate::setup::constants::LAUNCHER_NAME;

use crate::core::api_client::game::VersionDetails;
use crate::setup::constants::{APP_VERSION, LAUNCHER_NAME};
use crate::{
    core::{
        auth::Account,
        game::{GameInstance, GameJava},
        java::JavaRuntime,
    },
    setup::AppState,
};
use std::sync::Arc;

// const GAME_DIR: &str = "/Users/amagicpear/HMCL/.minecraft";
// const LIBRARY_PATH: &str = "/Users/amagicpear/HMCL/.minecraft/libraries";
// const ASSESTS_DIR: &str = "/Users/amagicpear/HMCL/.minecraft/assets";

/// Essential options for launching a Minecraft game
pub struct LaunchOption {
    account: Arc<Account>,
    java_runtime: Arc<JavaRuntime>,
    game_instance: Arc<GameInstance>,
    max_memory: usize,
    width: Option<usize>,
    height: Option<usize>,
}

impl LaunchOption {
    pub fn set_window_size(&mut self, width: usize, height: usize) -> &Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Launch a Minecraft game with the given options
    pub fn launch(&self) -> Result<std::process::Child, std::io::Error> {
        let mut command = std::process::Command::new(&self.java_runtime.java_exe);
        command
            .args(self.build_jvm_arguments()) // build jvm arguments
            .arg("-cp")
            .arg(self.build_classpath().unwrap_or_default())
            .arg("net.minecraft.client.main.Main")
            .args(self.build_game_arguments())
            .current_dir(&self.game_instance.global_dir.path);
        command.spawn()
    }

    fn build_jvm_arguments(&self) -> Vec<String> {
        let mut args = Vec::new();
        // memory setting
        args.push(format!("-Xmx{}m", self.max_memory));
        // encoding settings
        args.push("-Dfile.encoding=UTF-8".to_string());
        args.push("-Dstdout.encoding=UTF-8".to_string());
        args.push("-Dstderr.encoding=UTF-8".to_string());
        // safety settings
        args.push("-Djava.rmi.server.useCodebaseOnly=true".to_string());
        args.push("-Dcom.sun.jndi.rmi.object.trustURLCodebase=false".to_string());
        args.push("-Dcom.sun.jndi.cosnaming.object.trustURLCodebase=false".to_string());
        args.push("-Dlog4j2.formatMsgNoLookups=true".to_string());
        // dlog4j2 setup
        let log4j2_config = self.game_instance.directory.join("log4j2.xml");
        args.push(format!(
            "-Dlog4j.configurationFile={}",
            log4j2_config.display()
        ));
        // jar file
        args.push(format!(
            "-Dminecraft.client.jar={}",
            self.game_instance.jar_path.display()
        ));
        // macOS specific settings
        #[cfg(target_os = "macos")]
        {
            args.push("-XstartOnFirstThread".to_string());
            args.push("-Xdock:name=Minecraft".to_string());
            // TODO: 图标路径需要从 assets 中获取
        }
        // native libraries path
        let natives_path = &self.game_instance.natives_path.display();
        args.push(format!("-Djava.library.path={}", natives_path));
        args.push(format!("-Djna.tmpdir={}", natives_path));
        args.push(format!(
            "-Dorg.lwjgl.system.SharedLibraryExtractPath={}",
            natives_path
        ));
        args.push(format!("-Dio.netty.native.workdir={}", natives_path));
        // launcher info
        args.push(format!("-Dminecraft.launcher.brand={}", LAUNCHER_NAME));
        args.push(format!("-Dminecraft.launcher.version={}", APP_VERSION));
        // gc optimize
        args.append(
            &mut [
                "-XX:+UnlockExperimentalVMOptions",
                "-XX:+UnlockDiagnosticVMOptions",
                "-XX:+UseG1GC",
                "-XX:G1MixedGCCountTarget=5",
                "-XX:G1NewSizePercent=20",
                "-XX:G1ReservePercent=20",
                "-XX:MaxGCPauseMillis=50",
                "-XX:G1HeapRegionSize=32m",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
        );
        args
    }

    fn build_classpath(&self) -> Result<String, Box<dyn std::error::Error>> {
        let json_reader = std::fs::File::open(&self.game_instance.json_path)?;
        let version_json: VersionDetails = serde_json::from_reader(json_reader)?;
        let mut classpath = Vec::new();
        let libraries = version_json.libraries;
        for lib in libraries {
            let lib_artifact = lib.downloads.artifact;
            let lib_path = lib_artifact.path.ok_or("Missing path in artifact")?;
            let lib_full_path = format!(
                "{}/libraries/{}",
                self.game_instance.global_dir.path.display(),
                lib_path
            );
            classpath.push(lib_full_path);
        }
        if self.game_instance.jar_path.exists() {
            classpath.push(self.game_instance.jar_path.display().to_string());
        } else {
            return Err("Main jar file does not exist".into());
        }
        Ok(classpath.join(":"))
    }

    fn build_game_arguments(&self) -> Vec<String> {
        vec![
            format!("--username={}", (self.account.username())),
            format!("--version={}", self.game_instance.version),
            format!("--gameDir={}", self.game_instance.directory.display()),
            format!(
                "--assetsDir={}/assets",
                self.game_instance.global_dir.path.display()
            ),
            "--assetIndex=26".to_string(), // TODO: read from version json
            format!("--uuid={}", self.account.uuid()),
            // TODO: get the below from account
            format!(
                "--accessToken={}",
                self.account.access_token().unwrap_or("0")
            ),
            format!("--userType={}", "msa"),
            format!("--versionType={}", LAUNCHER_NAME),
            format!("--width={}", self.width.unwrap_or(854)),
            format!("--height={}", self.height.unwrap_or(480)),
        ]
    }

    /// build a launch option from app state if it is possible
    pub fn from_state(state: &AppState) -> Result<Self, Box<dyn std::error::Error>> {
        if let Some(game_instance) = state.active_game_instance.as_ref() {
            let java_selected: &Arc<JavaRuntime> = match game_instance.game_java {
                GameJava::Default => {
                    if let Some(java_runtime) = state.pcl_setup_info.default_java.as_ref() {
                        java_runtime
                    } else {
                        return Err("No default java runtime found".into());
                    }
                }
                GameJava::Custom(ref java_runtime) => java_runtime,
            };
            let active_account = state.active_account.as_ref();
            if active_account.is_none() {
                return Err("No active account found".into());
            }
            return Ok(Self {
                account: active_account.unwrap().clone(),
                java_runtime: java_selected.clone(),
                game_instance: game_instance.clone(),
                max_memory: state.pcl_setup_info.max_memory,
                width: None,
                height: None,
            });
        }
        Err("No active game instance found".into())
    }
}

#[test]
pub fn game_launch_test() {
    use crate::core::repository::GameRepository;
    use std::path::PathBuf;
    use std::sync::Arc;

    let account = Arc::new(Account::Offline {
        username: "AMagicPear".to_string(),
        uuid: "12345678-1234-1234-1234-123456789012".to_string(),
    });

    let game_repo = GameRepository::new("HMCL", PathBuf::from("/Users/amagicpear/HMCL/.minecraft"));
    let game_repo = Arc::new(game_repo);
    let version_folder = PathBuf::from("/Users/amagicpear/HMCL/.minecraft/versions/1.21.8");

    let mut launch_option = LaunchOption {
        account,
        java_runtime: Arc::new(JavaRuntime::try_from("/usr/bin/java").unwrap()),
        game_instance: Arc::new(
            GameInstance::from_version_folder(&version_folder, &game_repo).unwrap(),
        ),
        max_memory: 4096,
        width: None,
        height: None,
    };
    launch_option.set_window_size(1280, 720);
    if let Ok(mut child) = launch_option.launch() {
        child.wait().unwrap();
    } else {
        eprintln!(
            "launch failed, instance id:{:?}",
            launch_option.game_instance.id
        );
    }
}
