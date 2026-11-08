use crate::config::manager::ConfigManager;
use std::io::{Read, Write};

const MAX_BYTES: usize = 2 * 1024 * 1024;

fn read_homepage(path: &std::path::Path) -> Result<String, String> {
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|error| error.to_string())?
        .take((MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    decode_homepage(bytes)
}

fn decode_homepage(bytes: Vec<u8>) -> Result<String, String> {
    if bytes.len() > MAX_BYTES {
        return Err("Homepage exceeds 2 MiB".into());
    }
    String::from_utf8(bytes).map_err(|error| error.to_string())
}

#[tauri::command]
pub async fn load_custom_homepage(url: Option<String>) -> Result<String, String> {
    if let Some(url) = url {
        let url = reqwest::Url::parse(&url).map_err(|error| error.to_string())?;
        if !matches!(url.scheme(), "http" | "https")
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err("Invalid homepage URL".into());
        }
        use futures_util::StreamExt;
        let settings = ConfigManager::instance().api_client.resources().settings();
        let response = crate::config::manager::ConfigManager::instance()
            .api_client
            .transport()
            .send_request(
                settings.http_mode,
                false,
                url.as_str(),
                None,
                Some(std::time::Duration::from_secs(20)),
            )
            .await
            .map_err(|error| error.to_string())?
            .error_for_status()
            .map_err(|error| error.to_string())?;
        let mut bytes = Vec::new();
        let mut stream = response.bytes_stream();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk.map_err(|error| error.to_string())?;
            if bytes.len() + chunk.len() > MAX_BYTES {
                return Err("Homepage exceeds 2 MiB".into());
            }
            bytes.extend_from_slice(&chunk);
        }
        decode_homepage(bytes)
    } else {
        let path = ConfigManager::instance().config_dir().join("Custom.xaml");
        tauri::async_runtime::spawn_blocking(move || read_homepage(&path))
            .await
            .map_err(|error| error.to_string())?
    }
}

fn save_homepage(directory: &std::path::Path, content: &str) -> Result<(), String> {
    if content.len() > MAX_BYTES {
        return Err("Homepage exceeds 2 MiB".into());
    }
    std::fs::create_dir_all(directory).map_err(|error| error.to_string())?;
    let mut temporary =
        tempfile::NamedTempFile::new_in(directory).map_err(|error| error.to_string())?;
    temporary
        .write_all(content.as_bytes())
        .map_err(|error| error.to_string())?;
    temporary
        .as_file()
        .sync_all()
        .map_err(|error| error.to_string())?;
    temporary
        .persist(directory.join("Custom.xaml"))
        .map_err(|error| error.to_string())?;
    Ok(())
}

#[tauri::command]
pub async fn save_custom_homepage(content: String) -> Result<(), String> {
    let directory = ConfigManager::instance().config_dir().to_owned();
    tauri::async_runtime::spawn_blocking(move || save_homepage(&directory, &content))
        .await
        .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portable_reads_atomic_replacement_and_limits() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("Custom.xaml");
        let content = "<TextBlock Text=\"主页\" />";
        save_homepage(directory.path(), content).unwrap();
        save_homepage(directory.path(), "replacement").unwrap();
        assert_eq!(read_homepage(&path).unwrap(), "replacement");
        assert!(save_homepage(directory.path(), &"a".repeat(MAX_BYTES + 1)).is_err());
        assert_eq!(read_homepage(&path).unwrap(), "replacement");
        assert_eq!(std::fs::read_dir(directory.path()).unwrap().count(), 1);
        save_homepage(directory.path(), content).unwrap();
        assert_eq!(read_homepage(&path).unwrap(), content);
        std::fs::write(&path, vec![b'a'; MAX_BYTES + 1]).unwrap();
        assert!(read_homepage(&path).is_err());
        assert!(decode_homepage(vec![0xff]).is_err());
    }
}
