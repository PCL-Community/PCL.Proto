//! Java 列表的扫描编排、选择策略与持久化；探测与兼容性判断在 `pcl-core`。

use std::path::{Path, PathBuf};

use crate::config::java_preference::JavaLaunchInfo;
use crate::config::java_preference::current_instance;
use pcl_core::minecraft::instance::McInstance;
use pcl_core::minecraft::java::{JavaEntry, JavaInstallation, JavaManager, scan};
use std::time::{Duration, Instant};

use crate::config::manager::ConfigManager;
use crate::config::states::AppState;

/// 当前活动实例必须与预期一致，避免异步操作落到别的实例上。
pub fn active_instance<'a>(state: &'a AppState, expected: &Path) -> Result<&'a McInstance, String> {
    current_instance(state.active_game_instance.as_deref(), expected)
        .map_err(|error| error.to_string())
}

/// Installer candidates in host preference order; the core rechecks compatibility.
pub async fn installer_candidates(state: &tokio::sync::Mutex<AppState>) -> Vec<JavaEntry> {
    let (mut entries, selected) = {
        let state = state.lock().await;
        (
            state.java_runtimes.clone(),
            state.launcher_setup.selected_java.clone(),
        )
    };
    entries.sort_by(pcl_core::minecraft::java::candidate_order);
    if let Some(selected) = selected {
        if let Some(index) = entries.iter().position(|java| java.java_exe == selected) {
            let selected = entries.remove(index);
            entries.insert(0, selected);
        } else if let Ok(Ok(java)) =
            tokio::task::spawn_blocking(move || JavaEntry::probe(&selected)).await
        {
            entries.insert(0, java);
        }
    }
    entries
}

/// 重新扫描 Java 列表；结果写回配置。
pub async fn refresh(force: bool) -> Result<Vec<JavaEntry>, String> {
    let config = ConfigManager::instance();
    let (known, repositories, selected) = {
        let state = config.app_state.lock().await;
        (
            state.java_runtimes.clone(),
            state
                .repositories
                .iter()
                .map(|repository| repository.path.clone())
                .collect::<Vec<_>>(),
            state.launcher_setup.selected_java.clone(),
        )
    };
    let scanned = refresh_state(
        force,
        &JavaState {
            runtimes: known,
            selected,
        },
        &JavaRoots {
            repositories,
            runtime_dir: config.java_runtime_dir(),
        },
    )
    .await?;
    let runtimes = scanned.runtimes.clone();
    let selected = scanned.selected.clone();
    tokio::task::spawn_blocking(move || {
        config.update(|state| {
            state.java_runtimes = runtimes.clone();
            state.launcher_setup.selected_java = selected.clone();
            Ok(runtimes.clone())
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

/// 组装某个实例的 Java 启动信息；缺少运行时时会先刷新一次。
pub async fn prepare(expected: PathBuf) -> Result<JavaLaunchInfo, String> {
    async fn evaluate(expected: PathBuf) -> Result<JavaLaunchInfo, String> {
        let config = ConfigManager::instance();
        let (instance, runtimes, global, settings) = {
            let state = config.app_state.lock().await;
            let instance = active_instance(&state, &expected)
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
        let result = tokio::task::spawn_blocking(move || {
            let instance = instance
                .refresh()
                .map_err(|e| format!("Cannot refresh Minecraft instance: {e:?}"))?;
            JavaLaunchInfo::for_instance(&instance, &runtimes, global.as_deref(), &settings)
                .map_err(|e| e.to_string())
        })
        .await
        .map_err(|e| e.to_string())?;
        active_instance(&*config.app_state.lock().await, &expected).map_err(|e| e.to_string())?;
        result
    }
    let mut info = evaluate(expected.clone()).await?;
    if info.runtime.is_none() {
        refresh(false).await?;
        info = evaluate(expected).await?;
    }
    Ok(info)
}

static SCAN_LOCK: tokio::sync::Mutex<Option<Instant>> = tokio::sync::Mutex::const_new(None);

/// 已安装的 Java 与当前选择；由宿主应用持久化。
#[derive(Clone, Debug, Default)]
pub struct JavaState {
    pub runtimes: Vec<JavaEntry>,
    pub selected: Option<std::path::PathBuf>,
}

/// 扫描 Java 需要知道的目录：各 Minecraft 文件夹和启动器托管的运行时目录。
#[derive(Clone, Debug, Default)]
pub struct JavaRoots {
    pub repositories: Vec<std::path::PathBuf>,
    pub runtime_dir: std::path::PathBuf,
}

/// 重新扫描并合并 Java 列表；结果由宿主写回自己的配置。
async fn refresh_state(
    force: bool,
    state: &JavaState,
    roots: &JavaRoots,
) -> Result<JavaState, String> {
    let mut guard = SCAN_LOCK.lock().await;
    if !force && guard.is_some_and(|last| last.elapsed() < Duration::from_secs(13)) {
        return Ok(state.clone());
    }
    let mut directories = roots
        .repositories
        .iter()
        .map(|repository| repository.join("runtime"))
        .collect::<Vec<_>>();
    directories.push(roots.runtime_dir.clone());
    let known = state.runtimes.clone();
    if let Ok(directory) = crate::config::paths::launcher_directory() {
        directories.extend([
            directory.join("PCL"),
            directory.join("runtime"),
            directory.join("jre"),
            directory.join(".minecraft/runtime"),
        ]);
    }
    let scanned = scan(known.clone(), directories).await?;
    let selected = state.selected.clone();
    let selected_probe = selected.clone();
    let actual_selected = tokio::task::spawn_blocking(move || {
        selected_probe
            .and_then(|path| JavaInstallation::probe(&path).ok())
            .map(|installation| installation.java_exe)
    })
    .await
    .map_err(|error| error.to_string())?;
    let merged = tokio::task::spawn_blocking(move || {
        let mut manager = JavaManager::from_entries(known.clone());
        manager.merge_scanned(scanned, &known);
        manager.sorted()
    })
    .await
    .map_err(|error| error.to_string())?;
    *guard = Some(Instant::now());
    Ok(JavaState {
        runtimes: merged,
        selected: if selected.is_some() {
            actual_selected
        } else {
            None
        },
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn installer_reprobes_a_selected_java_missing_from_the_scan() {
        let root = std::env::temp_dir().join(format!("pcl-java-choice-{}", uuid::Uuid::new_v4()));
        let bin = root.join("bin");
        std::fs::create_dir_all(&bin).unwrap();
        let executable = bin.join(pcl_core::minecraft::java::java_binary());
        std::fs::write(&executable, b"fixture").unwrap();
        std::fs::write(
            root.join("release"),
            format!(
                "JAVA_VERSION=\"21.0.7\"\nOS_ARCH=\"{}\"\n",
                std::env::consts::ARCH
            ),
        )
        .unwrap();
        let mut state = AppState::default();
        state.launcher_setup.selected_java = Some(executable.clone());
        let state = tokio::sync::Mutex::new(state);
        let candidates = installer_candidates(&state).await;
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0].java_exe, executable.canonicalize().unwrap());
        std::fs::remove_dir_all(root).unwrap();
        assert!(installer_candidates(&state).await.is_empty());
    }
}
