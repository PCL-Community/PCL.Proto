use super::{JavaEntry, launcher_directory, requirements::JavaRequirements};

use crate::{app::config::AppState, minecraft::instance::McInstance};
use anyhow::{Context, ensure};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Default, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum JavaPreference {
    #[default]
    Global,
    Auto,
    #[serde(rename = "exist")]
    Existing {
        java_exe: PathBuf,
    },
    Relative {
        relative_path: PathBuf,
    },
}
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InstanceJavaSettings {
    pub preference: JavaPreference,
    pub ignore_compatibility: bool,
}
#[derive(Debug, Serialize)]
pub struct JavaLaunchInfo {
    pub instance_directory: PathBuf,
    pub requirements: JavaRequirements,
    pub runtime: Option<JavaEntry>,
    pub warnings: Vec<JavaWarning>,
    pub download_major: Option<u32>,
}
#[derive(Debug, Serialize)]
pub struct JavaWarning {
    pub key: String,
    pub version: String,
    pub min: String,
    pub max: String,
}
impl JavaLaunchInfo {
    pub fn for_instance(
        instance: &McInstance,
        runtimes: &[JavaEntry],
        global: Option<&Path>,
        settings: &InstanceJavaSettings,
    ) -> anyhow::Result<Self> {
        let requirements = JavaRequirements::for_instance(instance)?;
        let (runtime, warnings) = select(runtimes, global, settings, &requirements);
        let download_major = requirements.download_major();
        Ok(Self {
            instance_directory: instance.directory.clone(),
            runtime,
            warnings,
            download_major,
            requirements,
        })
    }
}
pub fn current_instance<'a>(
    state: &'a AppState,
    expected: &Path,
) -> anyhow::Result<&'a McInstance> {
    let instance = state
        .active_game_instance
        .as_deref()
        .context("No active Minecraft instance")?;
    ensure!(
        instance.directory == expected,
        "The active Minecraft instance changed; try again"
    );
    Ok(instance)
}

pub fn relative_java(root: &Path, relative: &Path) -> anyhow::Result<PathBuf> {
    ensure!(
        !relative.is_absolute() && !relative.as_os_str().is_empty(),
        "Java path must be relative to the launcher directory"
    );
    let root = root.canonicalize()?;
    let path = root.join(relative).canonicalize()?;
    ensure!(
        path.starts_with(&root),
        "Java path is outside the launcher directory"
    );
    ensure!(path.is_file(), "Java executable not found");
    Ok(path)
}

fn reprobe(path: &Path, known: &[JavaEntry]) -> Option<JavaEntry> {
    let mut actual = JavaEntry::probe(path)
        .map_err(|e| log::warn!("Java {} is unavailable: {e}", path.display()))
        .ok()?;
    if let Some(saved) = known
        .iter()
        .find(|j| j.java_exe == actual.java_exe || j.java_exe == path)
    {
        actual.is_enabled = saved.is_enabled;
        actual.source = saved.source;
    }
    (actual.is_enabled
        && !matches!(
            actual.installation.compatibility,
            crate::utils::platform::Compatibility::No
                | crate::utils::platform::Compatibility::Unknown
        ))
    .then_some(actual)
}

pub fn is_game_set_64_bit(
    known: &[JavaEntry],
    global: Option<&Path>,
    preference: &JavaPreference,
) -> bool {
    let any_64_bit = || known.iter().any(|j| j.is_64_bit);
    match preference {
        JavaPreference::Auto => any_64_bit(),
        JavaPreference::Existing { java_exe } => {
            JavaEntry::probe(java_exe).is_ok_and(|j| j.is_64_bit)
        }
        JavaPreference::Relative { relative_path } => launcher_directory()
            .and_then(|root| relative_java(&root, relative_path))
            .and_then(|path| JavaEntry::probe(&path))
            .is_ok_and(|j| j.is_64_bit),
        JavaPreference::Global => global
            .and_then(|path| JavaEntry::probe(path).ok())
            .map(|j| j.is_64_bit)
            .unwrap_or_else(any_64_bit),
    }
}

pub fn select(
    known: &[JavaEntry],
    global: Option<&Path>,
    settings: &InstanceJavaSettings,
    requirements: &JavaRequirements,
) -> (Option<JavaEntry>, Vec<JavaWarning>) {
    let mut warnings = Vec::new();
    let instance_path = match &settings.preference {
        JavaPreference::Existing { java_exe } => {
            Some((java_exe.clone(), "InstanceSelectedOutOfRange"))
        }
        JavaPreference::Relative { relative_path } => launcher_directory()
            .and_then(|root| relative_java(&root, relative_path))
            .ok()
            .map(|p| (p, "RelativePathSelectedOutOfRange")),
        _ => None,
    };
    // CE falls through to the global preference when an instance choice is absent or unusable.
    for (path, warning) in instance_path
        .into_iter()
        .chain(global.map(|p| (p.to_owned(), "GlobalSelectedOutOfRange")))
    {
        if let Some(runtime) = reprobe(&path, known) {
            if !settings.ignore_compatibility && !requirements.accepts(runtime.parsed_version()) {
                warnings.push(JavaWarning {
                    key: format!("Minecraft.Launch.Java.Compatibility.{warning}"),
                    version: runtime.version.clone(),
                    min: requirements.min.to_string(),
                    max: requirements.max.to_string(),
                });
            }
            return (Some(runtime), warnings);
        }
    }
    let mut candidates = known
        .iter()
        .filter(|j| j.is_enabled)
        .filter_map(|j| reprobe(&j.java_exe, known))
        .filter(|j| requirements.accepts(j.parsed_version()))
        .collect::<Vec<_>>();
    candidates.sort_by(|a, b| candidate_order(a, b));
    (candidates.into_iter().next(), warnings)
}

pub(super) fn candidate_order(a: &JavaEntry, b: &JavaEntry) -> std::cmp::Ordering {
    let rank = |j: &JavaEntry| match j.installation.compatibility {
        crate::utils::platform::Compatibility::Perfect => 0,
        _ => 1,
    };
    // Lowest acceptable major, then JDK, newest patch and path; vendor no longer ranks.
    a.major_version()
        .cmp(&b.major_version())
        .then(rank(a).cmp(&rank(b)))
        .then(b.is_jdk().cmp(&a.is_jdk()))
        .then(b.parsed_version().cmp(&a.parsed_version()))
        .then(a.java_exe.cmp(&b.java_exe))
}

pub fn validate_preference(preference: &JavaPreference) -> anyhow::Result<()> {
    match preference {
        JavaPreference::Existing { java_exe } => {
            ensure!(
                java_exe.is_absolute(),
                "Java executable must use an absolute path"
            );
        }
        JavaPreference::Relative { relative_path } => {
            relative_java(&launcher_directory()?, relative_path)?;
        }
        _ => (),
    }
    Ok(())
}

pub async fn prepare(expected: PathBuf) -> Result<JavaLaunchInfo, String> {
    async fn evaluate(expected: PathBuf) -> Result<JavaLaunchInfo, String> {
        let config = crate::app::config::ConfigManager::instance();
        let (instance, runtimes, global, settings) = {
            let state = config.app_state.lock().await;
            let instance = current_instance(&state, &expected)
                .map_err(|e| e.to_string())?
                .clone();
            let settings = state
                .java_preferences
                .get(&instance.directory)
                .cloned()
                .unwrap_or_default();
            (
                instance,
                state.java_runtimes.clone(),
                state.launcher_setup.selected_java.clone(),
                settings,
            )
        };
        let result = tauri::async_runtime::spawn_blocking(move || {
            let instance = instance
                .refresh()
                .map_err(|e| format!("Cannot refresh Minecraft instance: {e:?}"))?;
            JavaLaunchInfo::for_instance(&instance, &runtimes, global.as_deref(), &settings)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?;
        current_instance(&*config.app_state.lock().await, &expected).map_err(|e| e.to_string())?;
        result
    }
    let mut info = evaluate(expected.clone()).await?;
    if info.runtime.is_none() {
        super::refresh(false).await?;
        info = evaluate(expected).await?;
    }
    Ok(info)
}
