//! PCL 展示元数据留在应用层，核心实例类型不包含启动器私有配置。
use crate::config::{
    instance::{InstanceConfig, InstanceMetadata},
    manager::ConfigManager,
    states::AppState,
};
use base64::{Engine, engine::general_purpose};
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

fn known_instance(state: &AppState, path: &Path) -> Result<(), String> {
    if !path.is_absolute()
        || !path.is_dir()
        || std::fs::symlink_metadata(path)
            .map_err(|e| e.to_string())?
            .file_type()
            .is_symlink()
    {
        return Err("Invalid instance directory".into());
    }
    let parent = path.parent().ok_or("Missing versions directory")?;
    if !state
        .repositories
        .iter()
        .any(|repo| repo.path.join("versions") == parent)
    {
        return Err("Instance is outside configured Minecraft folders".into());
    }
    Ok(())
}

#[tauri::command]
pub async fn get_instance_metadata(
    directories: Vec<PathBuf>,
) -> Result<HashMap<PathBuf, InstanceMetadata>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = ConfigManager::instance().app_state.blocking_lock();
        directories
            .into_iter()
            .map(|path| {
                known_instance(&state, &path)?;
                let metadata = InstanceConfig::load(&path)?.metadata()?;
                Ok((path, metadata))
            })
            .collect()
    })
    .await
    .map_err(|error| error.to_string())?
}

#[derive(Default, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct MetadataPatch {
    starred: Option<bool>,
    display_type: Option<u8>,
    logo: Option<String>,
    logo_custom: Option<bool>,
}
impl MetadataPatch {
    fn apply(self, config: &mut InstanceConfig) -> Result<(), String> {
        if self.display_type.is_some_and(|value| value > 5)
            || self
                .logo
                .as_ref()
                .is_some_and(|value| value.len() > 16384 || value.contains('\0'))
        {
            return Err("Invalid instance metadata".into());
        }
        macro_rules! field {
            ($name:ident, $key:literal) => {
                if let Some(value) = self.$name {
                    config.set($key, value)?;
                }
            };
        }
        field!(starred, "IsStar");
        field!(display_type, "DisplayType");
        field!(logo, "Logo");
        field!(logo_custom, "LogoCustom");
        Ok(())
    }
}

#[tauri::command]
pub async fn set_instance_metadata(
    instance_directory: PathBuf,
    patch: MetadataPatch,
) -> Result<InstanceMetadata, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = ConfigManager::instance().app_state.blocking_lock();
        known_instance(&state, &instance_directory)?;
        InstanceConfig::update(&instance_directory, |file| {
            patch.apply(file)?;
            file.metadata()
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

fn image_data(path: &Path) -> Result<String, String> {
    use std::io::Read;
    let mut data = Vec::new();
    std::fs::File::open(path)
        .map_err(|e| e.to_string())?
        .take(10 * 1024 * 1024 + 1)
        .read_to_end(&mut data)
        .map_err(|e| e.to_string())?;
    if data.len() > 10 * 1024 * 1024 {
        return Err("Instance icon exceeds 10 MiB".into());
    }
    let mime = if data.starts_with(b"\x89PNG\r\n\x1a\n") {
        "image/png"
    } else if data.starts_with(b"\xff\xd8\xff") {
        "image/jpeg"
    } else if data.starts_with(b"GIF8") {
        "image/gif"
    } else if data.starts_with(b"RIFF") && data.get(8..12) == Some(b"WEBP") {
        "image/webp"
    } else {
        return Err("Unsupported instance icon format".into());
    };
    Ok(format!(
        "data:{mime};base64,{}",
        general_purpose::STANDARD.encode(data)
    ))
}

#[tauri::command]
pub async fn get_instance_custom_icon(instance_directory: PathBuf) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || {
        let state = ConfigManager::instance().app_state.blocking_lock();
        known_instance(&state, &instance_directory)?;
        let config = InstanceConfig::load(&instance_directory)?;
        let logo: String = config.get("Logo", String::new())?;
        // CE 使用相对实例目录的文件路径；跨平台统一识别两种分隔符。
        let relative = PathBuf::from(logo.replace('\\', "/"));
        if relative.is_absolute()
            || relative
                .components()
                .any(|part| !matches!(part, std::path::Component::Normal(_)))
        {
            return Err("Invalid relative instance icon path".into());
        }
        let root = instance_directory
            .canonicalize()
            .map_err(|e| e.to_string())?;
        let path = instance_directory
            .join(relative)
            .canonicalize()
            .map_err(|e| e.to_string())?;
        if !path.starts_with(root) {
            return Err("Instance icon is outside instance directory".into());
        }
        image_data(&path)
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn select_instance_custom_icon(
    app: tauri::AppHandle,
    instance_directory: PathBuf,
) -> Result<Option<InstanceMetadata>, String> {
    use tauri_plugin_dialog::DialogExt;
    tauri::async_runtime::spawn_blocking(move || {
        let selected = app
            .dialog()
            .file()
            .add_filter("Image", &["png", "jpg", "jpeg", "gif", "webp"])
            .blocking_pick_file();
        let Some(selected) = selected else {
            return Ok(None);
        };
        let selected = selected.into_path().map_err(|e| e.to_string())?;
        image_data(&selected)?;
        ConfigManager::instance().update_instance(&instance_directory, |config| {
            // 原子替换图片，随后更新同一份 CE 配置。
            let directory = instance_directory.join("PCL");
            std::fs::create_dir_all(&directory).map_err(|e| e.to_string())?;
            let mut image =
                tempfile::NamedTempFile::new_in(&directory).map_err(|e| e.to_string())?;
            let mut input = std::fs::File::open(&selected).map_err(|e| e.to_string())?;
            std::io::copy(&mut input, &mut image).map_err(|e| e.to_string())?;
            image
                .persist(directory.join("Logo.png"))
                .map_err(|e| e.to_string())?;
            config.set("Logo", "PCL\\Logo.png")?;
            config.set("LogoCustom", true)?;
            config.metadata().map(Some)
        })
    })
    .await
    .map_err(|error| error.to_string())?
}
