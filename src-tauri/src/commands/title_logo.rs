use base64::{Engine, engine::general_purpose::STANDARD};
use std::{fs, path::Path};

const MAX_LOGO_BYTES: u64 = 10 * 1024 * 1024;

fn logo_directory() -> Result<std::path::PathBuf, String> {
    Ok(crate::config::manager::ConfigManager::instance()
        .config_dir()
        .to_owned())
}

fn data_url(path: &Path) -> Result<String, String> {
    if fs::metadata(path).map_err(|error| error.to_string())?.len() > MAX_LOGO_BYTES {
        return Err("Title logo must be smaller than 10 MiB".into());
    }
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    let mime = match path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        _ => return Err("Unsupported title logo format".into()),
    };
    Ok(format!("data:{mime};base64,{}", STANDARD.encode(bytes)))
}

#[tauri::command]
pub fn get_title_logo() -> Result<Option<String>, String> {
    let directory = logo_directory()?;
    for extension in ["png", "jpg", "jpeg", "gif", "webp"] {
        let path = directory.join(format!("Logo.{extension}"));
        if path.is_file() {
            return data_url(&path).map(Some);
        }
    }
    Ok(None)
}

#[tauri::command]
pub fn set_title_logo(path: String) -> Result<String, String> {
    let source = Path::new(&path);
    let metadata = fs::metadata(source).map_err(|error| error.to_string())?;
    if !metadata.is_file() || metadata.len() > MAX_LOGO_BYTES {
        return Err("Title logo must be an image smaller than 10 MiB".into());
    }
    let extension = source
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "png" | "jpg" | "jpeg" | "gif" | "webp") {
        return Err("Unsupported title logo format".into());
    }
    let directory = logo_directory()?;
    let destination = directory.join(format!("Logo.{extension}"));
    let temporary = destination.with_extension("tmp");
    fs::copy(source, &temporary).map_err(|error| error.to_string())?;
    if destination.exists() {
        fs::remove_file(&destination).map_err(|error| error.to_string())?;
    }
    fs::rename(&temporary, &destination).map_err(|error| {
        let _ = fs::remove_file(&temporary);
        error.to_string()
    })?;
    for previous_extension in ["png", "jpg", "jpeg", "gif", "webp"] {
        let previous = directory.join(format!("Logo.{previous_extension}"));
        if previous != destination && previous.exists() {
            fs::remove_file(previous).map_err(|error| error.to_string())?;
        }
    }
    data_url(&destination)
}

#[tauri::command]
pub fn clear_title_logo() -> Result<(), String> {
    for extension in ["png", "jpg", "jpeg", "gif", "webp"] {
        let path = logo_directory()?.join(format!("Logo.{extension}"));
        if path.exists() {
            fs::remove_file(path).map_err(|error| error.to_string())?;
        }
    }
    Ok(())
}
