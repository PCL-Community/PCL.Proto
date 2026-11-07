//! Launcher log file management for the Setup - Log page, mirroring PCL CE.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tauri::{AppHandle, Manager};

use pcl_core::io::files::open_directory;

/// Name stem (without extension) of the log file this session writes to,
/// following CE's `Launch-yyyy-M-d-HHmmssfff` naming.
pub static LAUNCH_LOG_STEM: OnceLock<String> = OnceLock::new();

/// Same naming as the log plugin target configured in `lib.rs`.
pub fn launch_log_stem() -> String {
    LAUNCH_LOG_STEM
        .get_or_init(|| {
            let now = chrono::Local::now();
            format!(
                "Launch-{}-{}",
                now.format("%Y-%-m-%-d"),
                now.format("%H%M%S%3f")
            )
        })
        .clone()
}

/// One launcher log file inside the log directory.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogFileEntry {
    name: String,
    path: PathBuf,
    size: u64,
    /// Seconds since the Unix epoch, 0 when unavailable.
    modified: u64,
    /// The file the logger is currently writing to.
    current: bool,
}

/// The directory the log plugin writes to (`TargetKind::LogDir`).
fn log_directory(app: &AppHandle) -> Result<PathBuf, String> {
    app.path().app_log_dir().map_err(|error| error.to_string())
}

fn list_log_files_in(app: &AppHandle) -> Result<Vec<LogFileEntry>, String> {
    let directory = log_directory(app)?;
    let current = format!("{}.log", launch_log_stem());
    let mut entries = Vec::new();
    let readers = std::fs::read_dir(&directory).map_err(|error| error.to_string())?;
    for entry in readers {
        let entry = entry.map_err(|error| error.to_string())?;
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = entry.file_name().to_string_lossy().into_owned();
        let metadata = entry.metadata().map_err(|error| error.to_string())?;
        entries.push(LogFileEntry {
            size: metadata.len(),
            modified: metadata
                .modified()
                .ok()
                .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                .map(|duration| duration.as_secs())
                .unwrap_or(0),
            current: name == current,
            name,
            path,
        });
    }
    entries.sort_by(|a, b| b.modified.cmp(&a.modified).then(a.name.cmp(&b.name)));
    Ok(entries)
}

/// Log mutations must stay inside the log directory.
fn ensure_inside_log_directory(directory: &Path, path: &Path) -> Result<(), String> {
    let canonical_directory = directory
        .canonicalize()
        .map_err(|error| error.to_string())?;
    let canonical = path.canonicalize().map_err(|error| error.to_string())?;
    if !canonical.starts_with(&canonical_directory) {
        return Err("Log path is outside the log directory".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn list_log_files(app: AppHandle) -> Result<Vec<LogFileEntry>, String> {
    tauri::async_runtime::spawn_blocking(move || list_log_files_in(&app))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn open_log_directory(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let directory = log_directory(&app)?;
        std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
        open_directory(&directory)
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Open a log file with the system default program.
#[tauri::command]
pub async fn open_log_path(app: AppHandle, path: PathBuf) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let directory = log_directory(&app)?;
        ensure_inside_log_directory(&directory, &path)?;
        #[cfg(windows)]
        let result = open::that_detached(&path);
        #[cfg(not(windows))]
        let result = open::that(&path);
        result.map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Default export destination on the desktop, CE's `PCL_CE_Logs_<timestamp>.zip`.
#[tauri::command]
pub async fn log_export_destination(app: AppHandle) -> Result<String, String> {
    let directory = app
        .path()
        .desktop_dir()
        .or_else(|_| app.path().home_dir())
        .map_err(|error| error.to_string())?;
    let name = format!(
        "PCL_CE_Logs_{}.zip",
        chrono::Local::now().format("%Y%m%d%H%M%S")
    );
    Ok(directory.join(name).to_string_lossy().into_owned())
}

/// Pack the given log files into the selected zip archive.
#[tauri::command]
pub async fn export_log_files(
    app: AppHandle,
    paths: Vec<PathBuf>,
    destination: PathBuf,
) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let directory = log_directory(&app)?;
        let mut sources = Vec::with_capacity(paths.len());
        for path in &paths {
            ensure_inside_log_directory(&directory, path)?;
            sources.push(path.clone());
        }
        let file = std::fs::File::create(&destination).map_err(|error| error.to_string())?;
        let mut zip = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default();
        let packed = (|| -> Result<(), String> {
            for path in &sources {
                let name = path
                    .file_name()
                    .ok_or("Log path has no file name")?
                    .to_string_lossy()
                    .into_owned();
                zip.start_file(name, options)
                    .map_err(|error| error.to_string())?;
                let mut source = std::fs::File::open(path).map_err(|error| error.to_string())?;
                std::io::copy(&mut source, &mut zip).map_err(|error| error.to_string())?;
            }
            Ok(())
        })();
        let finished = zip.finish().map(|_| ()).map_err(|error| error.to_string());
        packed?;
        finished
    })
    .await
    .map_err(|error| error.to_string())?
}

/// Delete every log file except the one still being written to.
#[tauri::command]
pub async fn clear_log_files(app: AppHandle) -> Result<(), String> {
    tauri::async_runtime::spawn_blocking(move || {
        let entries = list_log_files_in(&app)?;
        let mut failures = Vec::new();
        for entry in &entries {
            if entry.current {
                continue;
            }
            if let Err(error) = std::fs::remove_file(&entry.path) {
                failures.push(format!("{}: {error}", entry.name));
            }
        }
        if failures.is_empty() {
            Ok(())
        } else {
            Err(failures.join("; "))
        }
    })
    .await
    .map_err(|error| error.to_string())?
}
