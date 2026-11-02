//! CE Java management, with executable probing and discovery for each desktop OS.
//!
//! The module layout follows `PCL.Core.Minecraft.Java`:
//! `installation`, `entry`, `manager`, scanners, parser, installer and
//! user-preference types.

use std::time::{Duration, Instant};

pub mod entry;
pub mod install;
pub mod installation;
pub mod manager;
pub mod preference;
pub mod requirements;
mod scanner;
#[cfg(test)]
mod tests;

#[allow(unused_imports)]
pub use crate::utils::platform::{Architecture, Compatibility};
#[allow(unused_imports)]
pub use entry::{JavaEntry, JavaSource, JavaStorageItem};
#[allow(unused_imports)]
pub use installation::{JavaInstallation, JavaVersion, java_binary, launcher_directory};
pub use manager::JavaManager;
pub(crate) use scanner::scan;

static SCAN_LOCK: tokio::sync::Mutex<Option<Instant>> = tokio::sync::Mutex::const_new(None);

pub async fn refresh(force: bool) -> Result<Vec<JavaEntry>, String> {
    let mut guard = SCAN_LOCK.lock().await;
    let config = crate::app::config::ConfigManager::instance();
    if !force && guard.is_some_and(|last| last.elapsed() < Duration::from_secs(13)) {
        return Ok(config.app_state.lock().await.java_runtimes.clone());
    }
    let (known, mut roots, selected) = {
        let state = config.app_state.lock().await;
        let known = state.java_runtimes.clone();
        let roots = state
            .repositories
            .iter()
            .map(|repository| repository.path.join("runtime"))
            .collect::<Vec<_>>();
        (known, roots, state.launcher_setup.selected_java.clone())
    };
    roots.push(config.java_runtime_dir());
    let scanned = scan(known.clone(), roots).await?;
    let selected_probe = selected.clone();
    let actual_selected = tokio::task::spawn_blocking(move || {
        selected_probe
            .and_then(|path| JavaInstallation::probe(&path).ok())
            .map(|installation| installation.java_exe)
    })
    .await
    .map_err(|error| error.to_string())?;
    let runtimes = tauri::async_runtime::spawn_blocking(move || {
        config.update(|state| {
            let mut manager = JavaManager::from_entries(std::mem::take(&mut state.java_runtimes));
            manager.merge_scanned(scanned, &known);
            state.java_runtimes = manager.sorted();
            if state.launcher_setup.selected_java == selected && actual_selected.is_some() {
                state.launcher_setup.selected_java = actual_selected;
            }
            Ok(state.java_runtimes.clone())
        })
    })
    .await
    .map_err(|error| error.to_string())??;
    *guard = Some(Instant::now());
    Ok(runtimes)
}

pub fn sort_list(list: &mut [JavaEntry]) {
    JavaManager::sort(list);
}
