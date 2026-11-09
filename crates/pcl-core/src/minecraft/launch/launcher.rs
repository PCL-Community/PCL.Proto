//! Mod for launching a Minecraft game
//! Code referenced from xphost/MMCLL/src/launcher.rs
//! MIT License | https://github.com/xphost008/MMCLL
//! Code referenced from scl-core/src/client.rs
//! LGPL-3.0 License & scl License | https://github.com/Steve-xmh/scl
//! Start script referenced from HMCL
//! GPL-3.0 License | https://github.com/HMCL-dev/HMCL

use super::resources;
use crate::io::net::http::api_client::game::McVersion;
use crate::minecraft::version::{McArgument, McArgumentValue, McRuleContext, rules_allow};
use crate::minecraft::{instance::McInstance, java::JavaEntry, profile::McProfile};
use std::sync::Arc;

/// `(自定义 JVM 参数, JVM 参数, 游戏参数)`，由 `metadata_arguments` 一次算好。
type ArgumentGroups = (Vec<String>, Vec<String>, Vec<String>);

/// 启动器身份：写进 `-Dminecraft.launcher.brand` 和脚本标题。
#[derive(Clone, Debug)]
pub struct LauncherBrand {
    pub name: String,
    pub version: String,
}

/// 启动辅助资源的替代路径；缺省时使用 core 自带资源。Mesa loader 需单独提供。
#[derive(Clone, Debug, Default)]
pub struct LaunchHelpers {
    pub java_wrapper: Option<std::path::PathBuf>,
    pub legacy_fix: Option<std::path::PathBuf>,
    pub lwjgl_unsafe_agent: Option<std::path::PathBuf>,
    pub log4j2_debug: Option<std::path::PathBuf>,
    pub log4j2_legacy_debug: Option<std::path::PathBuf>,
    pub macos_arm_libraries: Option<String>,
    pub mesa_loader: Option<std::path::PathBuf>,
}

/// 启动一次游戏所需的全部输入；由宿主应用从自己的配置组装。
pub struct LaunchRequest {
    pub wrapper_temp_dir: std::path::PathBuf,
    /// 默认启动辅助文件的输出目录，由宿主选择可写位置。
    pub helper_directory: std::path::PathBuf,
    pub brand: LauncherBrand,
    pub helpers: LaunchHelpers,
    pub instance: Arc<McInstance>,
    pub profile: McProfile,
    pub java: Arc<JavaEntry>,
    pub game_directory: std::path::PathBuf,
    pub max_memory: usize,
    pub initial_memory: Option<usize>,
    pub width: Option<usize>,
    pub height: Option<usize>,
    pub arguments: crate::minecraft::launch::arguments::LaunchArguments,
    pub extras: crate::minecraft::launch::settings::LaunchExtras,
}

/// Essential options for launching a Minecraft game
pub struct LaunchOption {
    wrapper_temp_dir: std::path::PathBuf,
    helper_directory: std::path::PathBuf,
    brand: LauncherBrand,
    helpers: LaunchHelpers,
    account: Arc<McProfile>,
    java_runtime: Arc<JavaEntry>,
    game_instance: Arc<McInstance>,
    game_directory: std::path::PathBuf,
    max_memory: usize,
    initial_memory: Option<usize>,
    width: Option<usize>,
    height: Option<usize>,
    version_details: McVersion,
    natives_path: std::path::PathBuf,
    arguments: crate::minecraft::launch::arguments::LaunchArguments,
    extras: crate::minecraft::launch::settings::LaunchExtras,
    server: Option<(String, Option<u16>)>,
    world: Option<String>,
    authlib: Option<AuthlibAgent>,
}

/// 第三方账号需要的 authlib-injector javaagent 参数，对照 CE 的 Authlib-Injector 分支。
pub struct AuthlibAgent {
    /// 本地 authlib-injector.jar 路径。
    pub jar: std::path::PathBuf,
    /// 认证服务器 API 根地址（不含 `/authserver`）。
    pub server: String,
    /// 服务器元数据的 Base64，作为 prefetched 传给 agent。
    pub prefetched: String,
}

impl AuthlibAgent {
    /// CE 把这三段拼成一个字符串塞进参数表；Proto 走 `Command::args`，必须拆开。
    pub fn arguments(&self) -> Vec<String> {
        let mut args = vec![
            format!("-javaagent:{}={}", self.jar.display(), self.server),
            "-Dauthlibinjector.side=client".to_owned(),
            format!("-Dauthlibinjector.yggdrasil.prefetched={}", self.prefetched),
        ];
        // CE：信任系统根证书（Meloong-Git/#5252）。WINDOWS-ROOT 只在 Windows 上存在。
        if cfg!(windows) {
            args.push("-Djavax.net.ssl.trustStoreType=WINDOWS-ROOT".to_owned());
        }
        args
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GameLaunchError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Json parsing failed: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Invalid Minecraft metadata: {0}")]
    InvalidMetadata(String),

    #[error("active account not found")]
    MissingAccount,

    #[error("invalid profile: {0}")]
    InvalidProfile(String),

    #[error("active game instance not found")]
    MissingInstance,

    #[error("Memory allocation failed: {0}")]
    Memory(String),

    #[error("Invalid custom launch arguments: {0}")]
    CustomArguments(String),

    #[error("Launcher resource error: {0}")]
    Resource(String),
}

impl LaunchOption {
    pub fn with_server(mut self, address: &str) -> Result<Self, GameLaunchError> {
        let (host, port) = crate::link::mcping::parse_address(address)
            .map_err(|error| GameLaunchError::CustomArguments(error.to_string()))?;
        self.server = Some((host, port));
        Ok(self)
    }

    pub fn with_world(mut self, world: &str) -> Result<Self, GameLaunchError> {
        let supported = self.version_details.arguments.as_ref().is_some_and(|args| {
            args.game.iter().any(|arg| match arg {
                McArgument::Plain(value) => value == "--quickPlaySingleplayer",
                McArgument::Conditional {
                    value: McArgumentValue::One(value),
                    ..
                } => value == "--quickPlaySingleplayer",
                McArgument::Conditional {
                    value: McArgumentValue::Many(values),
                    ..
                } => values
                    .iter()
                    .any(|value| value == "--quickPlaySingleplayer"),
            })
        });
        if !supported {
            return Err(GameLaunchError::InvalidMetadata(
                "This Minecraft version does not support singleplayer quick play".into(),
            ));
        }
        self.world = Some(world.to_owned());
        Ok(self)
    }

    /// 第三方账号在启动前准备好 javaagent 后挂上来。
    pub fn with_authlib(mut self, agent: AuthlibAgent) -> Self {
        self.authlib = Some(agent);
        self
    }

    pub fn account(&self) -> &McProfile {
        &self.account
    }

    pub fn with_window_size(mut self, width: usize, height: usize) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn launch_checked(&self) -> Result<std::process::Child, GameLaunchError> {
        // check account
        // check all the libraries
        self.launch()
    }

    /// Launch a Minecraft game with the given options
    fn launch(&self) -> Result<std::process::Child, GameLaunchError> {
        let mut command = self.command()?;
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            command.creation_flags(windows_sys::Win32::System::Threading::CREATE_NO_WINDOW);
        }
        crate::minecraft::launch::settings::run_prelaunch(
            &self.extras,
            &self.game_instance.global_dir,
        );
        Ok(command.spawn()?)
    }

    /// Build the java command without running it, so scripts can be exported.
    fn command(&self) -> Result<std::process::Command, GameLaunchError> {
        let mut command = std::process::Command::new(&self.java_runtime.java_exe);
        // Homebrew library overrides can crash AWT/AppKit while decoding application icons.
        #[cfg(target_os = "macos")]
        command.env_remove("DYLD_LIBRARY_PATH");
        let context = McRuleContext::for_java(&self.java_runtime.architecture)
            .map_err(|e| GameLaunchError::InvalidMetadata(e.to_string()))?;
        let head = self
            .arguments
            .classpath_paths()
            .map_err(GameLaunchError::CustomArguments)?;
        let classpath =
            build_classpath(&self.game_instance, &self.version_details, &context, &head)?;
        let (custom_jvm, mut jvm, game) = self.metadata_arguments(&classpath)?;
        let custom_initial = custom_jvm.iter().any(|arg| arg.starts_with("-Xms"));
        jvm.retain(|arg| {
            !arg.starts_with("-Xmx")
                && !(arg.starts_with("-Xms") && (custom_initial || self.initial_memory.is_some()))
        });
        let wrapper = self.java_wrapper()?;
        if let Some(agent) = &self.authlib {
            command.args(agent.arguments());
        }
        command
            .args(custom_jvm)
            .args(self.build_jvm_arguments(custom_initial)?)
            .args(jvm);
        if let Some(wrapper) = wrapper {
            command.args(wrapper.0).arg("-jar").arg(wrapper.1);
        }
        command
            .arg(&self.version_details.main_class)
            .args(game)
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .current_dir(&self.game_directory);
        Ok(command)
    }

    /// Write a launch script that reproduces this launch without the launcher.
    pub fn export_script(&self, path: &std::path::Path) -> Result<(), GameLaunchError> {
        let command = self.command()?;
        let args: Vec<String> = command
            .get_args()
            .map(|arg| arg.to_string_lossy().into_owned())
            .collect();
        let java = command.get_program().to_string_lossy().into_owned();
        let mut script = String::new();
        if cfg!(windows) {
            script.push_str("@echo off\r\n");
            script.push_str(&format!(
                "title {} - {}\r\n",
                self.brand.name, self.game_instance.name
            ));
            script.push_str("chcp 65001>nul\r\n");
            script.push_str(&format!("cd /D \"{}\"\r\n", self.game_directory.display()));
            if !self.extras.prelaunch_command.trim().is_empty() {
                script.push_str(self.extras.prelaunch_command.trim());
                script.push_str("\r\n");
            }
            script.push_str(&format!(
                "\"{}\" {}\r\n",
                java,
                args.iter()
                    .map(|arg| quote_windows(arg))
                    .collect::<Vec<_>>()
                    .join(" ")
            ));
            script.push_str("echo.\r\npause\r\n");
        } else {
            script.push_str("#!/bin/sh\n");
            script.push_str(&format!(
                "cd {} || exit 1\n",
                quote_unix(&self.game_directory.to_string_lossy())
            ));
            if !self.extras.prelaunch_command.trim().is_empty() {
                script.push_str(self.extras.prelaunch_command.trim());
                script.push('\n');
            }
            if cfg!(target_os = "macos") {
                script.push_str("unset DYLD_LIBRARY_PATH\n");
            }
            script.push_str(&quote_unix(&java));
            for arg in &args {
                script.push(' ');
                script.push_str(&quote_unix(arg));
            }
            script.push('\n');
        }
        let script = filter_access_token(script, self.account.access_token(), &self.account.uuid);
        let parent = path
            .parent()
            .ok_or_else(|| GameLaunchError::Resource("Invalid launch script path".into()))?;
        std::fs::create_dir_all(parent)?;
        let temp = path.with_extension("pcl-script-tmp");
        std::fs::write(&temp, script)?;
        std::fs::rename(&temp, path)?;
        Ok(())
    }

    pub fn access_token(&self) -> Option<String> {
        self.account.access_token().map(str::to_owned)
    }

    fn java_wrapper(&self) -> Result<Option<(Vec<String>, std::path::PathBuf)>, GameLaunchError> {
        // CE only wraps under a UTF-8 Windows code page, and the wrapper is off by default.
        if !self.extras.java_wrapper
            || !crate::minecraft::launch::settings::java_wrapper_supported()
        {
            return Ok(None);
        }
        let path = self.helper_path(
            self.helpers.java_wrapper.as_deref(),
            resources::JAVA_WRAPPER_NAME,
        )?;
        let mut args = Vec::new();
        if self.java_runtime.major_version() >= 9 {
            args.push("--add-exports".into());
            args.push("cpw.mods.bootstraplauncher/cpw.mods.bootstraplauncher=ALL-UNNAMED".into());
        }
        args.push(format!(
            "-Doolloo.jlw.tmpdir={}",
            self.wrapper_temp_dir.display()
        ));
        Ok(Some((args, path)))
    }

    fn helper_path(
        &self,
        supplied: Option<&std::path::Path>,
        name: &str,
    ) -> Result<std::path::PathBuf, GameLaunchError> {
        Ok(resources::helper_path(
            supplied,
            &self.helper_directory,
            name,
        )?)
    }

    fn build_jvm_arguments(&self, custom_initial: bool) -> Result<Vec<String>, GameLaunchError> {
        let mut args = Vec::new();
        // memory setting
        args.push(format!("-Xmx{}m", self.max_memory));
        if let Some(initial) = self.initial_memory.filter(|_| !custom_initial) {
            args.push(format!("-Xms{initial}m"));
        }
        // encoding settings
        args.push("-Dfile.encoding=UTF-8".to_string());
        args.push("-Dstdout.encoding=UTF-8".to_string());
        args.push("-Dstderr.encoding=UTF-8".to_string());
        // safety settings
        args.push("-Djava.rmi.server.useCodebaseOnly=true".to_string());
        args.push("-Dcom.sun.jndi.rmi.object.trustURLCodebase=false".to_string());
        args.push("-Dcom.sun.jndi.cosnaming.object.trustURLCodebase=false".to_string());
        args.push("-Dlog4j2.formatMsgNoLookups=true".to_string());
        if let Some(logging) = &self.version_details.logging {
            let path = self
                .game_instance
                .global_dir
                .join("assets/log_configs")
                .join(&logging.client.file.id);
            args.push(
                logging
                    .client
                    .argument
                    .replace("${path}", &path.to_string_lossy()),
            );
        }
        // jar file
        args.push(format!(
            "-Dminecraft.client.jar={}",
            self.game_instance.jar_path.display()
        ));
        // macOS specific settings
        #[cfg(target_os = "macos")]
        {
            // LWJGL 3 requests its main-thread mode in version metadata; LWJGL 2 uses AWT.
            args.push("-Xdock:name=Minecraft".to_string());
            // TODO: 图标路径需要从 assets 中获取
        }
        // native libraries path
        let natives_path = &self.natives_path.display();
        args.push(format!("-Djava.library.path={}", natives_path));
        args.push(format!("-Djna.tmpdir={}", natives_path));
        args.push(format!(
            "-Dorg.lwjgl.system.SharedLibraryExtractPath={}",
            natives_path
        ));
        args.push(format!("-Dio.netty.native.workdir={}", natives_path));
        // launcher info
        args.push(format!("-Dminecraft.launcher.brand={}", self.brand.name));
        args.push(format!(
            "-Dminecraft.launcher.version={}",
            self.brand.version
        ));
        args.extend(self.agent_arguments()?);
        Ok(args)
    }

    /// Java agents and JVM switches from the instance settings (CE order: agents last).
    fn agent_arguments(&self) -> Result<Vec<String>, GameLaunchError> {
        let mut args = Vec::new();
        if let Some((host, port)) = &self.extras.proxy {
            for scheme in ["http", "https"] {
                args.push(format!("-D{scheme}.proxyHost={host}"));
                args.push(format!("-D{scheme}.proxyPort={port}"));
            }
        }
        if self.extras.debug_log4j2 {
            let legacy = self.release_year().is_some_and(|year| year < 2017);
            let path = if legacy {
                self.helper_path(
                    self.helpers.log4j2_legacy_debug.as_deref(),
                    resources::LOG4J2_LEGACY_DEBUG_NAME,
                )?
            } else {
                self.helper_path(
                    self.helpers.log4j2_debug.as_deref(),
                    resources::LOG4J2_DEBUG_NAME,
                )?
            };
            args.push(format!("-Dlog4j.configurationFile={}", path.display()));
        }
        if self.extras.lwjgl_unsafe_agent && self.lwjgl_version().as_deref() == Some("3.4.1") {
            let path = self.helper_path(
                self.helpers.lwjgl_unsafe_agent.as_deref(),
                resources::LWJGL_UNSAFE_AGENT_NAME,
            )?;
            args.push(format!("-javaagent:{}", path.display()));
        }
        if self.extras.legacy_fix && self.needs_legacy_fix() {
            let path = self.helper_path(
                self.helpers.legacy_fix.as_deref(),
                resources::LEGACY_FIX_NAME,
            )?;
            args.push(format!("-javaagent:{}", path.display()));
            if self
                .version_details
                .release_time
                .as_deref()
                .is_some_and(|date| date < "2011-05-25")
            {
                args.push("-Djava.util.Arrays.useLegacyMergeSort=true".into());
            }
        }
        // CE only ships the renderer agent for Windows.
        if self.extras.renderer != 0 && cfg!(windows) {
            let path = self.helpers.mesa_loader.as_ref().ok_or_else(|| {
                GameLaunchError::Resource("Renderer agent is not available".into())
            })?;
            if !path.is_file() {
                return Err(GameLaunchError::Resource(
                    "Renderer agent is missing; start the game again to download it".into(),
                ));
            }
            let mode = match self.extras.renderer {
                1 => "llvmpipe",
                2 => "d3d12",
                _ => "zink",
            };
            args.push(format!("-javaagent:{}={mode}", path.display()));
        }
        Ok(args)
    }

    fn release_year(&self) -> Option<u32> {
        self.version_details
            .release_time
            .as_deref()
            .and_then(|time| time.get(..4))
            .and_then(|year| year.parse().ok())
    }

    fn needs_legacy_fix(&self) -> bool {
        self.version_details
            .release_time
            .as_deref()
            .is_some_and(|date| ("2000".."2013-06-25").contains(&date))
    }

    fn lwjgl_version(&self) -> Option<String> {
        self.version_details.libraries.iter().find_map(|library| {
            let mut parts = library.name.split(':');
            if parts.next()? != "org.lwjgl" || parts.next()? != "lwjgl" {
                return None;
            }
            parts.next().map(str::to_owned)
        })
    }

    fn metadata_arguments(&self, classpath: &str) -> Result<ArgumentGroups, GameLaunchError> {
        let instance = &self.game_instance;
        let assets = instance.global_dir.join("assets");
        let xuid = match self.account.as_ref() {
            profile
                if profile.profile_type == crate::minecraft::profile::ProfileType::Microsoft =>
            {
                profile.xuid().unwrap_or("")
            }
            _ => "0",
        };
        let mut vars = std::collections::HashMap::from([
            ("classpath", classpath.to_owned()),
            (
                "classpath_separator",
                if cfg!(windows) { ";" } else { ":" }.into(),
            ),
            (
                "natives_directory",
                self.natives_path.to_string_lossy().into_owned(),
            ),
            (
                "library_directory",
                instance
                    .global_dir
                    .join("libraries")
                    .to_string_lossy()
                    .into_owned(),
            ),
            (
                "libraries_directory",
                instance
                    .global_dir
                    .join("libraries")
                    .to_string_lossy()
                    .into_owned(),
            ),
            ("launcher_name", self.brand.name.clone()),
            ("launcher_version", self.brand.version.clone()),
            ("version_name", instance.name.clone()),
            ("version_type", self.extras.info.clone()),
            (
                "game_directory",
                self.game_directory.to_string_lossy().into_owned(),
            ),
            ("assets_root", assets.to_string_lossy().into_owned()),
            (
                "game_assets",
                assets.join("virtual/legacy").to_string_lossy().into_owned(),
            ),
            (
                "assets_index_name",
                self.version_details.asset_index.id.clone(),
            ),
            ("auth_player_name", self.account.user_name.clone()),
            ("auth_uuid", self.account.uuid.clone()),
            (
                "auth_access_token",
                self.account.access_token().unwrap_or("0").into(),
            ),
            (
                "access_token",
                self.account.access_token().unwrap_or("0").into(),
            ),
            (
                "auth_session",
                self.account.access_token().unwrap_or("0").into(),
            ),
            ("auth_xuid", xuid.into()),
            ("clientid", String::new()),
            ("user_type", "msa".into()),
            ("user_properties", "{}".into()),
            ("resolution_width", self.width.unwrap_or(854).to_string()),
            ("resolution_height", self.height.unwrap_or(480).to_string()),
        ]);
        let mut context = McRuleContext::for_java(&self.java_runtime.architecture)
            .map_err(|e| GameLaunchError::InvalidMetadata(e.to_string()))?;
        context
            .features
            .insert("has_custom_resolution".into(), true);
        if let Some(world) = &self.world {
            vars.insert("quickPlaySingleplayer", world.clone());
            context
                .features
                .insert("is_quick_play_singleplayer".into(), true);
        }
        let expand = |input: &str| -> Result<String, GameLaunchError> {
            let pattern = regex::Regex::new(r"\$\{([^}]+)\}").unwrap();
            let mut missing = None;
            let expanded = pattern.replace_all(input, |captures: &regex::Captures<'_>| {
                vars.get(&captures[1]).cloned().unwrap_or_else(|| {
                    missing = Some(captures[1].to_owned());
                    String::new()
                })
            });
            if let Some(name) = missing {
                return Err(GameLaunchError::InvalidMetadata(format!(
                    "Unknown argument placeholder: {name}"
                )));
            }
            Ok(expanded.into_owned())
        };
        let parse = |arguments: &[McArgument]| -> Result<Vec<String>, GameLaunchError> {
            let mut result = Vec::new();
            for argument in arguments {
                match argument {
                    McArgument::Plain(value) => result.push(expand(value)?),
                    McArgument::Conditional { rules, value }
                        if rules_allow(Some(rules), &context)
                            .map_err(|e| GameLaunchError::InvalidMetadata(e.to_string()))? =>
                    {
                        match value {
                            McArgumentValue::One(value) => result.push(expand(value)?),
                            McArgumentValue::Many(values) => {
                                for value in values {
                                    result.push(expand(value)?);
                                }
                            }
                        }
                    }
                    McArgument::Conditional { .. } => {}
                }
            }
            Ok(result)
        };
        let metadata = self.version_details.arguments.as_ref();
        let mut jvm = match metadata.filter(|a| !a.jvm.is_empty()) {
            Some(arguments) => parse(&arguments.jvm)?,
            None => Vec::new(),
        };
        if !jvm.iter().any(|arg| {
            matches!(arg.as_str(), "-cp" | "-classpath" | "--class-path")
                || arg.starts_with("--class-path=")
        }) {
            jvm.extend(["-cp".into(), classpath.into()]);
        }
        let mut game = if let Some(arguments) = metadata.filter(|a| !a.game.is_empty()) {
            parse(&arguments.game)?
        } else if let Some(legacy) = self.version_details.minecraft_arguments.as_deref() {
            let mut game = shlex::split(legacy)
                .ok_or_else(|| {
                    GameLaunchError::InvalidMetadata("Invalid legacy game arguments".into())
                })?
                .iter()
                .map(|argument| expand(argument))
                .collect::<Result<Vec<_>, _>>()?;
            if self
                .version_details
                .release_time
                .as_deref()
                .is_some_and(|date| date >= "2013-06-17")
                && !game
                    .iter()
                    .any(|arg| arg == "--height" || arg.starts_with("--height="))
            {
                game.extend([
                    "--height".into(),
                    self.height.unwrap_or(480).to_string(),
                    "--width".into(),
                    self.width.unwrap_or(854).to_string(),
                ]);
            }
            game
        } else {
            return Err(GameLaunchError::InvalidMetadata(
                "Missing Minecraft game arguments".into(),
            ));
        };
        let parse_custom = |value: &str| -> Result<Vec<String>, GameLaunchError> {
            shlex::split(value)
                .ok_or_else(|| GameLaunchError::CustomArguments("Unclosed quote or escape".into()))?
                .iter()
                .map(|argument| expand(argument))
                .collect()
        };
        let mut custom_jvm = parse_custom(&self.arguments.jvm)?;
        let last_initial = custom_jvm.iter().rposition(|arg| arg.starts_with("-Xms"));
        custom_jvm = custom_jvm
            .into_iter()
            .enumerate()
            .filter_map(|(index, arg)| {
                if arg.starts_with("-Xmx")
                    || (arg.starts_with("-Xms") && Some(index) != last_initial)
                {
                    None
                } else {
                    Some(arg)
                }
            })
            .collect();
        game.extend(parse_custom(&self.arguments.game)?);
        if self.extras.info.is_empty() {
            // CE drops the whole argument so the game falls back to the version's own type.
            // shortcut: only the vanilla two-token form is removed; `--versionType=` becomes empty.
            if let Some(index) = game.iter().position(|argument| argument == "--versionType") {
                game.drain(index..(index + 2).min(game.len()));
            }
        }
        if let Some((host, port)) = &self.server {
            if self
                .version_details
                .release_time
                .as_deref()
                .is_some_and(|date| date > "2023-04-04")
            {
                let address = match port {
                    Some(port) if host.contains(':') => format!("[{host}]:{port}"),
                    Some(port) => format!("{host}:{port}"),
                    None => host.clone(),
                };
                game.extend(["--quickPlayMultiplayer".into(), address]);
            } else {
                game.extend([
                    "--server".into(),
                    host.clone(),
                    "--port".into(),
                    port.unwrap_or(25565).to_string(),
                ]);
            }
        }
        Ok((custom_jvm, jvm, game))
    }

    /// 用宿主组装好的输入创建启动选项。
    pub fn new(request: LaunchRequest) -> Result<Self, GameLaunchError> {
        request
            .profile
            .validate_for_launch()
            .map_err(GameLaunchError::InvalidProfile)?;
        let game_instance = Arc::new(request.instance.refresh().map_err(|error| {
            GameLaunchError::InvalidMetadata(format!(
                "Cannot refresh Minecraft instance: {error:?}"
            ))
        })?);
        request
            .arguments
            .validate()
            .map_err(GameLaunchError::CustomArguments)?;
        Ok(Self {
            wrapper_temp_dir: request.wrapper_temp_dir,
            helper_directory: request.helper_directory,
            brand: request.brand,
            helpers: request.helpers,
            account: Arc::new(request.profile),
            java_runtime: request.java,
            version_details: game_instance.read_version_json()?,
            natives_path: game_instance.natives_path.clone(),
            game_instance,
            game_directory: request.game_directory,
            max_memory: request.max_memory,
            initial_memory: request.initial_memory,
            width: request.width,
            height: request.height,
            arguments: request.arguments,
            extras: request.extras,
            server: None,
            world: None,
            authlib: None,
        })
    }

    pub async fn prepare_natives(
        &mut self,
        downloader: &crate::io::download::Downloader,
    ) -> anyhow::Result<std::path::PathBuf> {
        if self.extras.renderer != 0 && cfg!(windows) {
            let path = self
                .helpers
                .mesa_loader
                .clone()
                .unwrap_or_else(|| resources::mesa_loader_path(&self.helper_directory));
            resources::ensure_mesa_loader(downloader, &path)
                .await
                .map_err(anyhow::Error::msg)?;
            self.helpers.mesa_loader = Some(path);
        }
        let context = McRuleContext::for_java(&self.java_runtime.architecture)?;
        patch_macos_arm_libraries(
            &mut self.version_details,
            &context,
            self.helpers.macos_arm_libraries.as_deref().or_else(|| {
                resources::embedded(resources::MACOS_ARM_LIBRARIES_NAME)
                    .and_then(|bytes| std::str::from_utf8(bytes).ok())
            }),
        )?;
        patch_macos_jna(&mut self.version_details, &context)?;
        let natives = crate::minecraft::installer::prepare_launch_libraries(
            &self.version_details,
            &self.game_instance.global_dir,
            &self.game_instance.directory,
            &context,
            downloader,
            !self.extras.disable_asset_verify,
        )
        .await?;
        self.natives_path = natives.clone();
        Ok(natives)
    }

    pub async fn prepare_legacy_assets(
        &self,
        downloader: &crate::io::download::Downloader,
    ) -> anyhow::Result<()> {
        crate::minecraft::installer::prepare_legacy_assets(
            &self.version_details,
            &self.game_instance.global_dir,
            &self.game_directory,
            downloader,
            !self.extras.disable_asset_verify,
        )
        .await
    }
}

fn patch_macos_jna(details: &mut McVersion, context: &McRuleContext) -> anyhow::Result<()> {
    if context.os != "osx" {
        return Ok(());
    }
    // JNA through 5.12 can abort when macOS returns a long dlopen error.
    let mut indices = Vec::new();
    for (index, library) in details.libraries.iter().enumerate() {
        if !library.rule_allow(context)? {
            continue;
        }
        let parts: Vec<_> = library.name.split(':').collect();
        if let ["net.java.dev.jna", name @ ("jna" | "jna-platform"), version] = parts.as_slice() {
            let Ok(version) = version
                .split('.')
                .map(str::parse::<u32>)
                .collect::<Result<Vec<_>, _>>()
            else {
                return Ok(());
            };
            if version.first() != Some(&5) || version.get(1).copied().unwrap_or(0) >= 13 {
                return Ok(());
            }
            indices.push((index, *name == "jna"));
        }
    }
    if indices.len() != 2
        || !indices.iter().any(|(_, core)| *core)
        || !indices.iter().any(|(_, core)| !*core)
    {
        return Ok(());
    }
    for (index, core) in indices {
        let name = if core { "jna" } else { "jna-platform" };
        let (sha1, size) = if name == "jna" {
            ("33d12735bef894440780fce64f9758d420c7bae2", 2002589)
        } else {
            ("a4934c44d25a9d8c2ddf4203affd20330cb3426f", 1378617)
        };
        let path = format!("net/java/dev/jna/{name}/5.17.0/{name}-5.17.0.jar");
        let library = &mut details.libraries[index];
        library.name = format!("net.java.dev.jna:{name}:5.17.0");
        library.url = None;
        library.sha1 = None;
        library.size = None;
        library.hint = None;
        library
            .downloads
            .get_or_insert_with(Default::default)
            .artifact = Some(crate::minecraft::version::DownloadFile {
            url: format!("https://libraries.minecraft.net/{path}"),
            path: Some(path),
            sha1: sha1.into(),
            size,
        });
    }
    Ok(())
}

fn patch_macos_arm_libraries(
    details: &mut McVersion,
    context: &McRuleContext,
    mapping: Option<&str>,
) -> anyhow::Result<()> {
    if context.os != "osx"
        || context.arch != "aarch64"
        || !details.libraries.iter().any(|library| {
            matches!(
                library.name.as_str(),
                "org.lwjgl:lwjgl:3.1.6"
                    | "org.lwjgl:lwjgl:3.2.1"
                    | "org.lwjgl.lwjgl:lwjgl-platform:2.9.1"
                    | "org.lwjgl.lwjgl:lwjgl-platform:2.9.1-nightly-20130708-debug3"
                    | "org.lwjgl.lwjgl:lwjgl-platform:2.9.2-nightly-20140822"
            )
        })
    {
        return Ok(());
    }
    // Library coordinates and checksums follow HMCL's macos-arm64 native mappings.
    let replacements: std::collections::HashMap<
        String,
        Option<crate::minecraft::version::McLibrary>,
    > = serde_json::from_str(mapping.ok_or_else(|| {
        anyhow::anyhow!(
            "The host must provide {}",
            resources::MACOS_ARM_LIBRARIES_NAME
        )
    })?)?;
    let mut libraries = Vec::new();
    for library in &details.libraries {
        if !library.rule_allow(context)? {
            libraries.push(library.clone());
            continue;
        }
        let artifact = replacements.get(&library.name);
        let native = library
            .natives
            .contains_key(&context.os)
            .then(|| replacements.get(&format!("{}:natives", library.name)))
            .flatten();
        if artifact.is_none() && native.is_none() {
            libraries.push(library.clone());
            continue;
        }
        let mut original = library.clone();
        if let Some(replacement) = artifact {
            if let Some(replacement) = replacement {
                libraries.push(replacement.clone());
            }
            if let Some(downloads) = &mut original.downloads {
                downloads.artifact = None;
            }
        }
        if let Some(replacement) = native {
            if let Some(replacement) = replacement {
                libraries.push(replacement.clone());
            }
            original.natives.clear();
            if let Some(downloads) = &mut original.downloads {
                downloads.classifiers.clear();
            }
        }
        if (artifact.is_none()
            && (original
                .downloads
                .as_ref()
                .is_some_and(|d| d.artifact.is_some())
                || (original.downloads.is_none() && library.natives.is_empty())))
            || !original.natives.is_empty()
        {
            libraries.push(original);
        }
    }
    details.libraries = libraries;
    Ok(())
}

#[cfg(test)]
mod macos_arm_patch_tests {
    use super::*;
    use serde_json::json;

    /// 宿主实际传入的映射来自应用资源；这里用等价结构验证替换规则。
    const MAPPING: &str = r#"{
        "org.lwjgl:lwjgl:3.2.1": {
            "name": "org.lwjgl:lwjgl:3.3.1",
            "downloads": {"artifact": {"sha1": "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa", "size": 1, "url": "https://example.org/lwjgl.jar"}}
        },
        "org.lwjgl:lwjgl:3.2.1:natives": {
            "name": "org.lwjgl:lwjgl:3.3.1:natives-macos-arm64",
            "downloads": {"classifiers": {"natives-macos-arm64": {"sha1": "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb", "size": 1, "url": "https://example.org/natives.jar"}}}
        },
        "org.lwjgl.lwjgl:lwjgl-platform:2.9.1:natives": {
            "name": "org.glavo.hmcl:lwjgl2-natives:2.9.3-rc1",
            "downloads": {"classifiers": {"natives-osx": {"sha1": "cccccccccccccccccccccccccccccccccccccccc", "size": 1, "url": "https://example.org/lwjgl2.jar"}}}
        },
        "net.java.jinput:jinput-platform:2.0.5": null,
        "net.java.jinput:jinput-platform:2.0.5:natives": null
    }"#;

    #[test]
    fn patches_legacy_libraries_only_for_arm_java() {
        let mut details: McVersion = serde_json::from_value(json!({
            "id":"test", "type":"release", "assets":"test", "mainClass":"Main",
            "downloads":{"client":{}}, "assetIndex":{"id":"test", "sha1":"", "size":1, "url":""},
            "libraries":[
                {"name":"org.lwjgl:lwjgl:3.2.1", "natives":{"osx":"natives-macos"}, "downloads":{"artifact":{}, "classifiers":{"natives-macos":{}}}},
                {"name":"org.lwjgl.lwjgl:lwjgl-platform:2.9.1", "natives":{"osx":"natives-osx"}, "downloads":{"classifiers":{"natives-osx":{}}}},
                {"name":"net.java.jinput:jinput-platform:2.0.5", "natives":{"osx":"natives-osx"}, "downloads":{"classifiers":{"natives-osx":{}}}}
            ]
        })).unwrap();
        let mut context = McRuleContext {
            os: "osx".into(),
            arch: "x86_64".into(),
            version: String::new(),
            features: Default::default(),
        };
        patch_macos_arm_libraries(&mut details, &context, Some(MAPPING)).unwrap();
        assert_eq!(details.libraries[0].name, "org.lwjgl:lwjgl:3.2.1");
        context.arch = "aarch64".into();
        patch_macos_arm_libraries(&mut details, &context, Some(MAPPING)).unwrap();
        let names: Vec<_> = details
            .libraries
            .iter()
            .map(|lib| lib.name.as_str())
            .collect();
        assert_eq!(
            names,
            [
                "org.lwjgl:lwjgl:3.3.1",
                "org.lwjgl:lwjgl:3.3.1:natives-macos-arm64",
                "org.glavo.hmcl:lwjgl2-natives:2.9.3-rc1"
            ]
        );
        for library in &details.libraries {
            if let Some(downloads) = &library.downloads {
                for info in downloads
                    .artifact
                    .iter()
                    .chain(downloads.classifiers.values())
                {
                    info.validate().unwrap();
                }
            }
        }
        patch_macos_arm_libraries(&mut details, &context, Some(MAPPING)).unwrap();
        assert_eq!(details.libraries.len(), 3);
    }
}

/// Batch tokens break on whitespace and shell metacharacters; quote those arguments.
fn quote_windows(arg: &str) -> String {
    // CE 语义：参数已经自带结尾引号时不再二次包裹，否则含空格、盘符或 cmd 元字符才补引号。
    let needs_quotes = arg.contains(' ')
        || arg.contains(":\\")
        || arg
            .chars()
            .any(|c| matches!(c, '\t' | ';' | '&' | '|' | '<' | '>' | '^' | '(' | ')'));
    if needs_quotes && !arg.ends_with('"') {
        format!("\"{arg}\"")
    } else {
        arg.to_owned()
    }
}

/// 对齐 CE 的 `McLogFilter.FilterAccessToken`：保留前后各 5 个字符，中间用 `F` 打码。
/// UUID 与 token 相同（离线登录等场景）时不打码。
fn filter_access_token(script: String, token: Option<&str>, uuid: &str) -> String {
    let Some(token) = token.filter(|token| token.len() >= 10 && *token != uuid) else {
        return script;
    };
    let chars: Vec<char> = token.chars().collect();
    let masked: String = chars[..5]
        .iter()
        .copied()
        .chain(std::iter::repeat_n('F', chars.len() - 10))
        .chain(chars[chars.len() - 5..].iter().copied())
        .collect();
    script.replace(token, &masked)
}

fn quote_unix(arg: &str) -> String {
    format!("'{}'", arg.replace('\'', "'\\''"))
}

fn build_classpath(
    instance: &McInstance,
    details: &McVersion,
    context: &McRuleContext,
    head: &[std::path::PathBuf],
) -> Result<String, GameLaunchError> {
    let mut classpath: Vec<_> = head.iter().rev().cloned().collect();
    let mut seen = std::collections::HashSet::new();
    for lib in &details.libraries {
        if !lib
            .rule_allow(context)
            .map_err(|e| GameLaunchError::InvalidMetadata(e.to_string()))?
        {
            continue;
        }
        let Some(path) = lib
            .classpath_path(&instance.global_dir, &instance.directory)
            .map_err(|e| GameLaunchError::InvalidMetadata(e.to_string()))?
        else {
            continue;
        };
        let key = lib
            .classpath_key()
            .map_err(|e| GameLaunchError::InvalidMetadata(e.to_string()))?;
        if !seen.insert(key) {
            continue;
        }
        if !path.is_file() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Missing library: {}", path.display()),
            )
            .into());
        }
        classpath.push(path);
    }
    if !instance.jar_path.is_file() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Minecraft client JAR not found",
        )
        .into());
    }
    classpath.push(instance.jar_path.clone());
    let joined = std::env::join_paths(classpath)
        .map_err(|e| GameLaunchError::InvalidMetadata(e.to_string()))?;
    joined
        .into_string()
        .map_err(|_| GameLaunchError::InvalidMetadata("Classpath is not valid Unicode".into()))
}

/// 测试里构造启动请求的最小输入。
#[cfg(test)]
pub(crate) fn test_request(
    instance: Arc<McInstance>,
    profile: McProfile,
    java: Arc<JavaEntry>,
    game_directory: std::path::PathBuf,
) -> LaunchRequest {
    LaunchRequest {
        wrapper_temp_dir: std::env::temp_dir(),
        helper_directory: game_directory.join("launcher-resources"),
        brand: LauncherBrand {
            name: "PCL.Proto".into(),
            version: "test".into(),
        },
        helpers: LaunchHelpers::default(),
        instance,
        profile,
        java,
        game_directory,
        max_memory: 2048,
        initial_memory: None,
        width: None,
        height: None,
        arguments: crate::minecraft::launch::arguments::LaunchArguments::default(),
        extras: Default::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;

    #[test]
    fn upgrades_old_jna_pair_only_on_macos() {
        let artifact =
            json!({"sha1":"a".repeat(40), "size":1, "url":"https://example.org/client.jar"});
        let mut details: McVersion = serde_json::from_value(json!({
            "id":"test", "type":"release", "assets":"test", "mainClass":"Main",
            "downloads":{"client":artifact}, "assetIndex":{"id":"test","sha1":"a".repeat(40),"size":1,"url":"https://example.org/index"},
            "libraries":[{"name":"net.java.dev.jna:jna:5.12.1"}, {"name":"net.java.dev.jna:jna-platform:5.12.1"}]
        })).unwrap();
        let mut context = McRuleContext::current();
        context.os = "linux".into();
        patch_macos_jna(&mut details, &context).unwrap();
        assert_eq!(details.libraries[0].name, "net.java.dev.jna:jna:5.12.1");
        context.os = "osx".into();
        patch_macos_jna(&mut details, &context).unwrap();
        assert_eq!(details.libraries[0].name, "net.java.dev.jna:jna:5.17.0");
        assert_eq!(
            details.libraries[1].name,
            "net.java.dev.jna:jna-platform:5.17.0"
        );
        assert!(details.libraries.iter().all(|lib| {
            lib.downloads
                .as_ref()
                .unwrap()
                .artifact
                .as_ref()
                .unwrap()
                .validate()
                .is_ok()
        }));
    }

    #[cfg(unix)]
    #[test]
    fn metadata_arguments_reach_the_process_without_shell_splitting() {
        use std::os::unix::fs::PermissionsExt;
        let root = std::env::temp_dir().join(format!("pcl-launch-test-{}", uuid::Uuid::new_v4()));
        let version = root.join("versions/Custom Name");
        fs::create_dir_all(&version).unwrap();
        fs::write(version.join("Custom Name.jar"), b"client").unwrap();
        let artifact =
            json!({"sha1":"a".repeat(40), "size":1, "url":"https://example.org/client.jar"});
        let metadata = json!({
            "id":"Custom Name", "type":"release", "assets":"test", "mainClass":"Main",
            "downloads":{"client":artifact}, "assetIndex":{"id":"test","sha1":"a".repeat(40),"size":1,"url":"https://example.org/index"}, "libraries":[],
            "arguments":{
                "jvm":["-cp","${classpath}", "-Dnative.fixture=${natives_directory}", {"rules":[{"action":"allow","os":{"arch":"aarch64"}}],"value":"-Darm=true"}, {"rules":[{"action":"allow","features":{"has_custom_resolution":true}}],"value":["-Dwidth=${resolution_width}","-Dlauncher=${launcher_name}"]}, {"rules":[{"action":"allow","features":{"is_demo_user":true}}],"value":"-Ddemo"}],
                "game":["--username","${auth_player_name}","--gameDir","${game_directory}","--accessToken","${auth_access_token}"]
            }
        });
        fs::write(
            version.join("Custom Name.json"),
            serde_json::to_vec(&metadata).unwrap(),
        )
        .unwrap();
        let executable = root.join("fake-java");
        fs::write(
            &executable,
            "#!/bin/sh\noutput=\"$(dirname \"$0\")/captured\"\npwd > \"$output\"\nprintf '%s\\n' \"$@\" >> \"$output\"\n",
        )
        .unwrap();
        fs::set_permissions(&executable, fs::Permissions::from_mode(0o755)).unwrap();
        let repo = crate::minecraft::instance::folder::McFolder::new("test", root.clone());
        let instance = Arc::new(McInstance::from_version_folder(&version, &repo).unwrap());
        let runtime: JavaEntry = serde_json::from_value(json!({
            "java_folder":root, "version":"21", "slug_version":21,
            "is_64_bit":true, "architecture":"Arm64", "compatibility":"Perfect",
            "java_exe":executable, "vendor":"OpenJDK"
        }))
        .unwrap();
        let mut option = LaunchOption {
            wrapper_temp_dir: std::env::temp_dir(),
            helper_directory: root.join("launcher-resources"),
            brand: LauncherBrand {
                name: "PCL.Proto".into(),
                version: "test".into(),
            },
            helpers: LaunchHelpers::default(),
            account: Arc::new(McProfile::yggdrasil(
                "Name With Space",
                "uuid",
                "secret token",
            )),
            java_runtime: Arc::new(runtime),
            game_instance: instance.clone(),
            game_directory: instance.directory.clone(),
            max_memory: 2048,
            initial_memory: None,
            width: Some(1024),
            height: Some(768),
            version_details: instance.read_version_json().unwrap(),
            natives_path: instance.natives_path.clone(),
            arguments: crate::minecraft::launch::arguments::LaunchArguments::default(),
            extras: Default::default(),
            server: None,
            world: None,
            authlib: None,
        };
        option.extras.proxy = Some(("127.0.0.1".into(), 7891));
        let proxy_args = option.agent_arguments().unwrap();
        assert!(proxy_args.contains(&"-Dhttp.proxyHost=127.0.0.1".into()));
        assert!(proxy_args.contains(&"-Dhttps.proxyPort=7891".into()));
        assert!(!proxy_args.iter().any(|arg| arg.starts_with("-javaagent")));
        option.extras.proxy = None;
        assert!(!option.helper_directory.exists());
        option.extras.debug_log4j2 = true;
        let log_config = option.helper_directory.join(resources::LOG4J2_DEBUG_NAME);
        if let Some(bytes) = resources::embedded(resources::LOG4J2_DEBUG_NAME) {
            assert!(option.agent_arguments().unwrap().contains(&format!(
                "-Dlog4j.configurationFile={}",
                log_config.display()
            )));
            assert_eq!(fs::read(&log_config).unwrap(), bytes);
            fs::remove_file(&log_config).unwrap();
        } else {
            assert!(option.agent_arguments().is_err());
            assert!(!log_config.exists());
        }
        let supplied = root.join("custom-log.xml");
        fs::write(&supplied, b"custom").unwrap();
        option.helpers.log4j2_debug = Some(supplied.clone());
        assert!(
            option
                .agent_arguments()
                .unwrap()
                .contains(&format!("-Dlog4j.configurationFile={}", supplied.display()))
        );
        assert!(!log_config.exists());
        assert_eq!(fs::read(&supplied).unwrap(), b"custom");
        option.helpers.log4j2_debug = None;
        fs::create_dir_all(&option.helper_directory).unwrap();
        fs::create_dir(&log_config).unwrap();
        assert!(option.agent_arguments().is_err());
        fs::remove_dir(&log_config).unwrap();
        option.extras.debug_log4j2 = false;
        option.version_details.release_time = Some("2013-06-24T00:00:00Z".into());
        assert!(option.needs_legacy_fix());
        option.version_details.release_time = Some("2013-06-25T00:00:00Z".into());
        assert!(!option.needs_legacy_fix());
        option.version_details.release_time = Some("1999-12-31T00:00:00Z".into());
        assert!(!option.needs_legacy_fix());
        option.version_details.release_time = None;
        option.version_details.libraries = serde_json::from_value(json!([
            {"name":"org.lwjgl:lwjgl:3.4.1"},
            {"name":"example:other:1.0"}
        ]))
        .unwrap();
        assert_eq!(option.lwjgl_version().as_deref(), Some("3.4.1"));
        option.version_details.libraries.clear();
        option.extras.info = "Custom Info".into();
        let original_arguments = option.version_details.arguments.take();
        assert!(
            !option
                .build_jvm_arguments(false)
                .unwrap()
                .iter()
                .any(|arg| arg == "-XstartOnFirstThread")
        );
        option.version_details.arguments = Some(
            serde_json::from_value(json!({"jvm":["-XstartOnFirstThread"], "game":["--username","${auth_player_name}"]})).unwrap(),
        );
        #[cfg(target_os = "macos")]
        {
            assert!(
                option
                    .command()
                    .unwrap()
                    .get_envs()
                    .any(|(key, value)| key == "DYLD_LIBRARY_PATH" && value.is_none())
            );
            let script_path = root.join("launch.command");
            option.export_script(&script_path).unwrap();
            assert!(
                fs::read_to_string(script_path)
                    .unwrap()
                    .contains("unset DYLD_LIBRARY_PATH\n")
            );
        }
        assert_eq!(
            option
                .command()
                .unwrap()
                .get_args()
                .filter(|arg| *arg == "-XstartOnFirstThread")
                .count(),
            1
        );
        option.version_details.arguments = Some(
            serde_json::from_value(json!({"jvm":["-Djna.tmpdir=${natives_directory}"],"game":["--versionType","${version_type}"]}))
                .unwrap(),
        );
        let (_, jvm, game) = option.metadata_arguments("a path").unwrap();
        assert!(jvm.contains(&format!("-Djna.tmpdir={}", option.natives_path.display())));
        assert_eq!(game, ["--versionType", "Custom Info"]);
        option.extras.info = String::new();
        let (_, _, game) = option.metadata_arguments("a path").unwrap();
        assert!(game.is_empty(), "{game:?}");
        option.extras.info = "Custom Info".into();
        option = option.with_server("[::1]:25566").unwrap();
        option.version_details.release_time = Some("2024-01-01T00:00:00Z".into());
        let (_, _, game) = option.metadata_arguments("a path").unwrap();
        assert!(
            game.windows(2)
                .any(|args| args == ["--quickPlayMultiplayer", "[::1]:25566"])
        );
        option.version_details.release_time = Some("2020-01-01T00:00:00Z".into());
        let (_, _, game) = option.metadata_arguments("a path").unwrap();
        assert!(
            game.windows(4)
                .any(|args| args == ["--server", "::1", "--port", "25566"])
        );
        option.server = None;
        option.version_details.release_time = None;
        option.version_details.arguments = Some(serde_json::from_value(json!({
            "game": ["--username", "${auth_player_name}", {
                "rules": [{"action": "allow", "features": {"is_quick_play_singleplayer": true}}],
                "value": ["--quickPlaySingleplayer", "${quickPlaySingleplayer}"]
            }]
        })).unwrap());
        option = option.with_world("A world").unwrap();
        let (_, _, game) = option.metadata_arguments("a path").unwrap();
        assert!(
            game.windows(2)
                .any(|args| args == ["--quickPlaySingleplayer", "A world"])
        );
        option.world = None;
        option.version_details.arguments = original_arguments;
        assert!(option.launch_checked().unwrap().wait().unwrap().success());
        let output = fs::read_to_string(root.join("captured")).unwrap();
        let lines: Vec<_> = output.lines().collect();
        assert_eq!(
            fs::canonicalize(lines[0]).unwrap(),
            fs::canonicalize(&version).unwrap()
        );
        assert!(lines.contains(&"-Dwidth=1024"));
        assert!(lines.contains(&"-Darm=true"));
        assert!(
            lines.contains(
                &format!("-Dnative.fixture={}", instance.natives_path.display()).as_str()
            )
        );
        assert!(!lines.contains(&"-Ddemo"));
        assert!(
            lines
                .windows(2)
                .any(|pair| pair == ["--username", "Name With Space"])
        );
        assert!(
            lines
                .windows(2)
                .any(|pair| pair == ["--gameDir", version.to_str().unwrap()])
        );
        assert!(
            lines
                .windows(2)
                .any(|pair| pair == ["--accessToken", "secret token"])
        );
        // 未隔离的实例使用全局目录。
        let shared = LaunchOption::new(test_request(
            instance.clone(),
            option.account.as_ref().clone(),
            option.java_runtime.clone(),
            root.clone(),
        ))
        .unwrap();
        assert!(shared.launch_checked().unwrap().wait().unwrap().success());
        let shared_output = fs::read_to_string(root.join("captured")).unwrap();
        let shared_lines: Vec<_> = shared_output.lines().collect();
        assert_eq!(
            fs::canonicalize(shared_lines[0]).unwrap(),
            fs::canonicalize(&root).unwrap()
        );
        assert!(
            shared_lines
                .windows(2)
                .any(|pair| pair == ["--gameDir", root.to_str().unwrap()])
        );
        // 宿主已解析的内存和参数交给实际进程。
        let first_head = root.join("first extra.jar");
        let second_head = root.join("second.jar");
        let instance_arguments = crate::minecraft::launch::arguments::LaunchArguments {
            jvm: "-XX:+UseZGC -Xmx8G -Xms256m -Xms512m -Dfixture='two words' -Droot=${game_directory}".into(),
            game: "--server 'my host' --asset ${assets_root}".into(),
            classpath: format!("{};{}", first_head.display(), second_head.display()),
        };
        let mut request = test_request(
            instance.clone(),
            option.account.as_ref().clone(),
            option.java_runtime.clone(),
            root.clone(),
        );
        request.max_memory = 3072;
        request.initial_memory = Some(1024);
        request.arguments = instance_arguments;
        let custom = LaunchOption::new(request).unwrap();
        assert!(custom.launch_checked().unwrap().wait().unwrap().success());
        let output = fs::read_to_string(root.join("captured")).unwrap();
        let lines: Vec<_> = output.lines().collect();
        assert!(lines.contains(&"-XX:+UseZGC"));
        assert!(!lines.contains(&"-XX:+UseG1GC"));
        assert!(!lines.contains(&"-Dglobal=true"));
        assert!(lines.contains(&"-Dfixture=two words"));
        assert!(lines.contains(&format!("-Droot={}", root.display()).as_str()));
        assert_eq!(
            lines
                .iter()
                .filter(|arg| arg.starts_with("-Xmx"))
                .copied()
                .collect::<Vec<_>>(),
            ["-Xmx3072m"]
        );
        assert_eq!(
            lines
                .iter()
                .filter(|arg| arg.starts_with("-Xms"))
                .copied()
                .collect::<Vec<_>>(),
            ["-Xms512m"]
        );
        assert!(lines.windows(2).any(|pair| pair == ["--server", "my host"]));
        assert!(
            lines
                .windows(2)
                .any(|pair| pair == ["--asset", root.join("assets").to_str().unwrap()])
        );
        let cp = lines.windows(2).find(|pair| pair[0] == "-cp").unwrap()[1];
        assert_eq!(
            std::env::split_paths(cp).take(2).collect::<Vec<_>>(),
            [second_head, first_head]
        );
        option.version_details.arguments = Some(
            serde_json::from_value(
                json!({"jvm":["-Dloader=true"],"game":["--username","${auth_player_name}"]}),
            )
            .unwrap(),
        );
        assert_eq!(
            option.metadata_arguments("a path").unwrap().1,
            ["-Dloader=true", "-cp", "a path"]
        );
        option.version_details.arguments = None;
        option.version_details.minecraft_arguments = Some(
            "--username ${auth_player_name} --title 'Legacy Game' --assetsDir ${assets_root}"
                .into(),
        );
        let (_, jvm, game) = option.metadata_arguments("a path").unwrap();
        assert_eq!(jvm, ["-cp", "a path"]);
        assert_eq!(game[1], "Name With Space");
        assert_eq!(game[3], "Legacy Game");
        option.version_details.release_time = Some("2013-06-17T00:00:00Z".into());
        let (_, _, game) = option.metadata_arguments("a path").unwrap();
        assert!(game.windows(2).any(|pair| pair == ["--width", "1024"]));
        assert!(game.windows(2).any(|pair| pair == ["--height", "768"]));
        assert!(option.launch_checked().unwrap().wait().unwrap().success());
        let captured = fs::read_to_string(root.join("captured")).unwrap();
        let lines: Vec<_> = captured.lines().collect();
        assert!(lines.windows(2).any(|pair| pair == ["--width", "1024"]));
        assert!(lines.windows(2).any(|pair| pair == ["--height", "768"]));
        option.version_details.release_time = Some("2013-06-16T00:00:00Z".into());
        assert!(
            !option
                .metadata_arguments("a path")
                .unwrap()
                .2
                .contains(&"--width".into())
        );
        option.version_details.minecraft_arguments = Some("--title 'unclosed".into());
        assert!(option.metadata_arguments("a path").is_err());
        option.version_details.minecraft_arguments = Some("${unknown}".into());
        assert!(
            option
                .metadata_arguments("a path")
                .unwrap_err()
                .to_string()
                .contains("unknown")
        );
        let child = root.join("versions/Fabric");
        fs::create_dir_all(&child).unwrap();
        fs::write(
            child.join("Fabric.json"),
            serde_json::to_vec(&json!({
                "id":"Fabric", "inheritsFrom":"Custom Name", "mainClass":"FabricMain",
                "arguments":{"jvm":["-Dloader=true"],"game":[]},
                "libraries":[{"name":"example:loader:1", "url":"https://example.org/"}]
            }))
            .unwrap(),
        )
        .unwrap();
        fs::create_dir_all(root.join("libraries/example/loader/1")).unwrap();
        let loader = root.join("libraries/example/loader/1/loader-1.jar");
        fs::write(&loader, b"loader").unwrap();
        let inherited = Arc::new(McInstance::from_version_folder(&child, &repo).unwrap());
        option.version_details = inherited.read_version_json().unwrap();
        option.game_directory = inherited.directory.clone();
        option.game_instance = inherited;
        assert!(option.launch_checked().unwrap().wait().unwrap().success());
        let captured = fs::read_to_string(root.join("captured")).unwrap();
        let lines: Vec<_> = captured.lines().collect();
        assert_eq!(
            fs::canonicalize(lines[0]).unwrap(),
            fs::canonicalize(&child).unwrap()
        );
        assert!(lines.contains(&"FabricMain"));
        assert!(lines.contains(&"-Dloader=true"));
        assert!(
            lines
                .windows(2)
                .any(|pair| pair == ["--username", "Name With Space"])
        );
        let cp = lines.windows(2).find(|pair| pair[0] == "-cp").unwrap()[1];
        assert_eq!(
            std::env::split_paths(cp).collect::<Vec<_>>(),
            [loader, version.join("Custom Name.jar")]
        );
        fs::write(child.join("Fabric.jar"), b"patched").unwrap();
        let forge_metadata = json!({
            "id":"Fabric", "inheritsFrom":"Custom Name", "clientVersion":"1.20.1", "mainClass":"ForgeMain",
            "libraries":[{"name":"net.minecraftforge:forge:1.20.1-47.0.0"}]
        });
        fs::write(
            child.join("Fabric.json"),
            serde_json::to_vec(&forge_metadata).unwrap(),
        )
        .unwrap();
        let forge_library =
            root.join("libraries/net/minecraftforge/forge/1.20.1-47.0.0/forge-1.20.1-47.0.0.jar");
        fs::create_dir_all(forge_library.parent().unwrap()).unwrap();
        fs::write(&forge_library, b"forge").unwrap();
        let mut request = test_request(
            option.game_instance.clone(),
            option.account.as_ref().clone(),
            option.java_runtime.clone(),
            child.clone(),
        );
        request.width = Some(1280);
        request.height = Some(720);
        let custom = LaunchOption::new(request).unwrap();
        assert!(custom.launch_checked().unwrap().wait().unwrap().success());
        assert!(
            fs::read_to_string(root.join("captured"))
                .unwrap()
                .lines()
                .any(|line| line == "-Dwidth=1280")
        );
        let option = LaunchOption::new(test_request(
            option.game_instance.clone(),
            option.account.as_ref().clone(),
            option.java_runtime.clone(),
            child.clone(),
        ))
        .unwrap();
        assert_eq!(option.game_instance.jar_path, child.join("Fabric.jar"));
        assert!(option.launch_checked().unwrap().wait().unwrap().success());
        let captured = fs::read_to_string(root.join("captured")).unwrap();
        let lines: Vec<_> = captured.lines().collect();
        let cp = lines.windows(2).find(|pair| pair[0] == "-cp").unwrap()[1];
        assert_eq!(
            std::env::split_paths(cp).collect::<Vec<_>>(),
            [forge_library, child.join("Fabric.jar")]
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn launch_script_quotes_paths_and_arguments() {
        assert_eq!(
            quote_windows("-Djava.library.path=/tmp/a"),
            "-Djava.library.path=/tmp/a"
        );
        assert_eq!(
            quote_windows("-cp C:\\a b\\lib.jar;C:\\c.jar"),
            "\"-cp C:\\a b\\lib.jar;C:\\c.jar\""
        );
        assert_eq!(
            quote_windows("-Dlog4j.configurationFile=\"C:\\path\\cfg.xml\""),
            "-Dlog4j.configurationFile=\"C:\\path\\cfg.xml\""
        );
        assert_eq!(
            quote_unix("/tmp/with space/it's.jar"),
            "'/tmp/with space/it'\\''s.jar'"
        );
        let token = "abcdefghijklmnopqrstuvwxyz";
        let masked = filter_access_token(
            format!("-cp . --accessToken {token}"),
            Some(token),
            "another-uuid",
        );
        assert_eq!(masked, "-cp . --accessToken abcdeFFFFFFFFFFFFFFFFvwxyz");
        assert_eq!(masked.len(), format!("-cp . --accessToken {token}").len());
        assert_eq!(
            filter_access_token(format!("--accessToken {token}"), Some(token), token),
            format!("--accessToken {token}")
        );
    }

    #[test]
    fn classpath_uses_install_rules_and_reports_missing_libraries() {
        let root =
            std::env::temp_dir().join(format!("pcl-classpath-test-{}", uuid::Uuid::new_v4()));
        let version = root.join("versions/Custom");
        fs::create_dir_all(&version).unwrap();
        fs::create_dir_all(root.join("libraries/test")).unwrap();
        fs::write(root.join("libraries/test/allowed.jar"), b"library").unwrap();
        fs::write(version.join("Custom.jar"), b"client").unwrap();
        let artifact = |path| json!({"sha1":"a".repeat(40), "size":1,"url":"https://example.org/library.jar","path":path});
        let metadata = json!({
            "id":"Custom", "clientVersion":"1.test", "type":"release", "assets":"test", "mainClass":"Main",
            "downloads":{"client":artifact("client.jar")},
            "assetIndex":{"id":"test","sha1":"a".repeat(40),"size":1,"url":"https://example.org/index.json"},
            "libraries":[
                {"name":"test:allowed:1","downloads":{"artifact":artifact("test/allowed.jar")}},
                {"name":"test:blocked:1","rules":[{"action":"disallow"}],"downloads":{"artifact":artifact("test/absent.jar")}},
                {"name":"test:native:1","downloads":{"classifiers":{}}}
            ]
        });
        fs::write(
            version.join("Custom.json"),
            serde_json::to_vec(&metadata).unwrap(),
        )
        .unwrap();
        let repo = crate::minecraft::instance::folder::McFolder::new("test", root.clone());
        let instance = McInstance::from_version_folder(&version, &repo).unwrap();
        let details = instance.read_version_json().unwrap();
        let classpath =
            build_classpath(&instance, &details, &McRuleContext::current(), &[]).unwrap();
        assert_eq!(
            std::env::split_paths(&classpath).collect::<Vec<_>>(),
            vec![
                root.join("libraries/test/allowed.jar"),
                version.join("Custom.jar")
            ]
        );
        fs::remove_file(root.join("libraries/test/allowed.jar")).unwrap();
        assert!(
            build_classpath(&instance, &details, &McRuleContext::current(), &[])
                .unwrap_err()
                .to_string()
                .contains("Missing library")
        );
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn inherited_classpath_prefers_child_and_accepts_local_maven_metadata() {
        let root =
            std::env::temp_dir().join(format!("pcl-inherited-classpath-{}", uuid::Uuid::new_v4()));
        let parent = root.join("versions/base");
        let child = root.join("versions/loader");
        fs::create_dir_all(&parent).unwrap();
        fs::create_dir_all(&child).unwrap();
        fs::write(parent.join("base.jar"), b"client").unwrap();
        let artifact = |path| json!({"sha1":"a".repeat(40), "size":1,"url":"https://example.org/file","path":path});
        fs::write(parent.join("base.json"), serde_json::to_vec(&json!({
            "id":"base","type":"release","mainClass":"Main","assets":"test",
            "downloads":{"client":artifact("client.jar")},"assetIndex":{"id":"test","sha1":"a".repeat(40),"size":1,"url":"https://example.org/index"},
            "libraries":[{"name":"example:shared:1","downloads":{"artifact":artifact("missing-old.jar")}}, {"name":"example:other:1","rules":[{"action":"allow"}],"downloads":{"artifact":artifact("other.jar")}}]
        })).unwrap()).unwrap();
        fs::write(
            child.join("loader.json"),
            serde_json::to_vec(&json!({"id":"loader","inheritsFrom":"base","libraries":[
                {"name":"example:shared:2"},
                {"name":"example:local:1","downloads":{"artifact":{"path":"local.jar"}}},
                {"name":"example:other:2","rules":[{"action":"disallow"}]}
            ]}))
            .unwrap(),
        )
        .unwrap();
        fs::create_dir_all(root.join("libraries/example/shared/2")).unwrap();
        fs::write(
            root.join("libraries/example/shared/2/shared-2.jar"),
            b"loader",
        )
        .unwrap();
        fs::write(root.join("libraries/other.jar"), b"parent").unwrap();
        fs::write(root.join("libraries/local.jar"), b"local").unwrap();
        let instance = McInstance::from_version_folder(
            &child,
            &crate::minecraft::instance::folder::McFolder::new("test", root.clone()),
        )
        .unwrap();
        let classpath = build_classpath(
            &instance,
            &instance.read_version_json().unwrap(),
            &McRuleContext::current(),
            &[],
        )
        .unwrap();
        assert_eq!(
            std::env::split_paths(&classpath).collect::<Vec<_>>(),
            vec![
                root.join("libraries/example/shared/2/shared-2.jar"),
                root.join("libraries/local.jar"),
                root.join("libraries/other.jar"),
                parent.join("base.jar")
            ]
        );
        fs::remove_file(root.join("libraries/example/shared/2/shared-2.jar")).unwrap();
        assert!(
            build_classpath(
                &instance,
                &instance.read_version_json().unwrap(),
                &McRuleContext::current(),
                &[]
            )
            .unwrap_err()
            .to_string()
            .contains("Missing library")
        );
        fs::remove_dir_all(root).unwrap();
    }
}
