use crate::{io::download::Downloader, minecraft::version::DownloadFile};
use anyhow::{Context, ensure};
use std::path::{Path, PathBuf};

fn urls(loader: &str, game: &str, version: &str) -> anyhow::Result<(String, String)> {
    for value in [game, version] {
        ensure!(
            !value.is_empty()
                && value.len() <= 100
                && value
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._+-".contains(&b))
                && value != "."
                && value != "..",
            "Invalid version"
        );
    }
    match loader {
        "forge" => {
            let id = format!("{game}-{version}");
            Ok((
                format!(
                    "https://maven.minecraftforge.net/net/minecraftforge/forge/{id}/forge-{id}-installer.jar"
                ),
                format!(
                    "https://files.minecraftforge.net/maven/net/minecraftforge/forge/{id}/forge-{id}-changelog.txt"
                ),
            ))
        }
        "fabric" | "legacyfabric" => Ok((
            format!(
                "https://maven.fabricmc.net/net/fabricmc/fabric-loader/{version}/fabric-loader-{version}.jar"
            ),
            "https://fabricmc.net/blog".into(),
        )),
        _ => anyhow::bail!("Unsupported loader"),
    }
}

struct TempFile(PathBuf);
impl Drop for TempFile {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn publish(temp: &Path, destination: &Path) -> anyhow::Result<()> {
    std::fs::hard_link(temp, destination)
        .context("Cannot publish download (the destination may already exist)")?;
    Ok(())
}

pub async fn save_loader_file(
    downloader: &Downloader,
    loader: String,
    game: String,
    version: String,
    destination: PathBuf,
) -> Result<(), String> {
    async {
        let (url, _) = urls(&loader, &game, &version)?;
        let parent = destination
            .parent()
            .filter(|p| p.is_absolute() && p.is_dir())
            .context("Invalid destination")?;
        let name = destination
            .file_name()
            .and_then(|name| name.to_str())
            .context("Invalid destination")?;
        crate::io::files::validate_file_name(name)?;
        ensure!(!destination.try_exists()?, "Destination already exists");
        let client = downloader.transport().http_client();
        let sha1 = client
            .get(format!("{url}.sha1"))
            .send()
            .await?
            .error_for_status()?
            .text()
            .await?
            .trim()
            .to_owned();
        let size = client
            .head(&url)
            .send()
            .await?
            .error_for_status()?
            .content_length()
            .context("Missing file size")?;
        ensure!(size > 0 && size <= 64 * 1024 * 1024, "Invalid file size");
        let temp = TempFile(parent.join(format!(".pcl-loader-{}.tmp", uuid::Uuid::new_v4())));
        downloader
            .download(
                &DownloadFile {
                    sha1,
                    size,
                    url,
                    path: None,
                },
                Path::new(&temp.0),
            )
            .await?;
        publish(&temp.0, &destination)?;
        Ok::<(), anyhow::Error>(())
    }
    .await
    .map_err(|error| format!("{error:#}"))
}

/// 加载器更新日志的地址；打开浏览器由宿主完成。
pub fn changelog_url(loader: &str, game: &str, version: &str) -> Result<String, String> {
    let (_, url) = urls(loader, game, version).map_err(|e| e.to_string())?;
    Ok(url)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn loader_urls_are_bounded_to_known_sources() {
        assert!(
            urls("forge", "1.20.1", "47.2.0")
                .unwrap()
                .0
                .ends_with("forge-1.20.1-47.2.0-installer.jar")
        );
        assert!(
            urls("fabric", "1.20.1", "0.16.10")
                .unwrap()
                .0
                .ends_with("fabric-loader-0.16.10.jar")
        );
        assert!(urls("forge", "../bad", "47").is_err());
        assert!(urls("unknown", "1.20.1", "47").is_err());
    }

    #[test]
    fn publication_never_overwrites_an_existing_file() {
        let dir = tempfile::tempdir().unwrap();
        let temp = dir.path().join("download.tmp");
        let destination = dir.path().join("installer.jar");
        std::fs::write(&temp, b"download").unwrap();
        std::fs::write(&destination, b"existing").unwrap();
        assert!(publish(&temp, &destination).is_err());
        assert_eq!(std::fs::read(&destination).unwrap(), b"existing");
        std::fs::remove_file(&destination).unwrap();
        publish(&temp, &destination).unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"download");
    }
}
