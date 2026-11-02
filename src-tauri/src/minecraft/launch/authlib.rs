//! authlib-injector 准备与启动参数，对照 CE 的 `ModLaunch` / `ModLibrary` 中的
//! Authlib-Injector 部分。
//!
//! 第三方账号必须靠 javaagent 把游戏的认证请求转到皮肤站，否则登录成功也进不了服务器。

use std::path::{Path, PathBuf};

use base64::Engine;
use sha2::Digest;

use crate::minecraft::launch::launcher::AuthlibAgent;
use crate::minecraft::profile::McProfile;

/// CE 的下载源顺序：官方源优先，失败换 BMCLAPI 镜像。
const METADATA_SOURCES: [&str; 2] = [
    "https://authlib-injector.yushi.moe/artifact/latest.json",
    "https://bmclapi2.bangbang93.com/mirrors/authlib-injector/artifact/latest.json",
];

/// 第三方档案启动前的准备：确保 jar 存在，并预取认证服务器元数据。
pub async fn prepare(profile: &McProfile) -> Result<AuthlibAgent, String> {
    let server = profile
        .server
        .as_deref()
        .map(|server| {
            server
                .replace("/authserver", "")
                .trim_end_matches('/')
                .to_owned()
        })
        .filter(|server| !server.is_empty())
        .ok_or("第三方档案没有认证服务器地址")?;
    let client =
        crate::minecraft::profile::auth::http_client().map_err(|error| error.to_string())?;
    let jar = ensure_jar(&client).await?;
    // CE 把服务器根地址的元数据 Base64 后交给 agent，避免游戏启动时再请求一次。
    let metadata = client
        .get(&server)
        .send()
        .await
        .map_err(|_| format!("无法连接认证服务器：{server}"))?
        .text()
        .await
        .map_err(|_| format!("无法连接认证服务器：{server}"))?;
    if metadata.trim().is_empty() {
        return Err(format!("无法连接认证服务器：{server}"));
    }
    Ok(AuthlibAgent {
        jar,
        prefetched: base64::engine::general_purpose::STANDARD.encode(metadata.as_bytes()),
        server,
    })
}

/// authlib-injector.jar 的存放位置，对照 CE 的 `pathPure`。
fn jar_path() -> PathBuf {
    crate::app::config::ConfigManager::instance()
        .config_dir()
        .join("authlib-injector.jar")
}

/// 按 latest.json 的 sha256 校验本地 jar，需要时重新下载。
///
/// 元数据取不到时沿用已有 jar（与 CE 一致：下载信息失败只记日志）。
async fn ensure_jar(client: &reqwest::Client) -> Result<PathBuf, String> {
    let path = jar_path();
    let latest = fetch_latest(client).await;
    let expected = match &latest {
        Some((_, sha256)) => sha256.clone(),
        None => {
            return path
                .exists()
                .then_some(path)
                .ok_or_else(|| "无法获取 Authlib-Injector 下载信息，且本地没有缓存".to_owned());
        }
    };
    if sha256_of(&path) == Some(expected.clone()) {
        return Ok(path);
    }
    let (urls, _) = latest.unwrap();
    download(client, &urls, &path, &expected).await?;
    Ok(path)
}

async fn fetch_latest(client: &reqwest::Client) -> Option<(Vec<String>, String)> {
    for source in METADATA_SOURCES {
        let Ok(response) = client.get(source).send().await else {
            continue;
        };
        let Ok(metadata) = response.json::<serde_json::Value>().await else {
            continue;
        };
        let Some(sha256) = metadata
            .pointer("/checksums/sha256")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        let Some(download_url) = metadata
            .get("download_url")
            .and_then(serde_json::Value::as_str)
        else {
            continue;
        };
        // CE：主源失败时换成镜像地址再试一次。
        let mirror = download_url.replace(
            "authlib-injector.yushi.moe",
            "bmclapi2.bangbang93.com/mirrors/authlib-injector",
        );
        let official = download_url.replace(
            "bmclapi2.bangbang93.com/mirrors/authlib-injector",
            "authlib-injector.yushi.moe",
        );
        let mut urls = vec![official];
        if mirror != urls[0] {
            urls.push(mirror);
        }
        return Some((urls, sha256.to_owned()));
    }
    None
}

async fn download(
    client: &reqwest::Client,
    urls: &[String],
    path: &Path,
    expected: &str,
) -> Result<(), String> {
    let mut last_error = "没有可用的 Authlib-Injector 下载地址".to_owned();
    for url in urls {
        match download_once(client, url, path).await {
            Ok(()) => {
                if sha256_of(path).as_deref() == Some(expected) {
                    return Ok(());
                }
                let _ = std::fs::remove_file(path);
                last_error = "Authlib-Injector 校验失败".to_owned();
            }
            Err(error) => last_error = error,
        }
    }
    Err(last_error)
}

async fn download_once(client: &reqwest::Client, url: &str, path: &Path) -> Result<(), String> {
    let response = client
        .get(url)
        .send()
        .await
        .map_err(|error| format!("下载 Authlib-Injector 失败：{error}"))?;
    if !response.status().is_success() {
        return Err(format!(
            "下载 Authlib-Injector 失败：HTTP {}",
            response.status()
        ));
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|error| format!("下载 Authlib-Injector 失败：{error}"))?;
    // 先写临时文件再改名，避免半截文件被当成有效缓存。
    let temp = path.with_extension("jar.downloading");
    tokio::fs::write(&temp, &bytes)
        .await
        .map_err(|error| error.to_string())?;
    tokio::fs::rename(&temp, path)
        .await
        .map_err(|error| error.to_string())?;
    Ok(())
}

fn sha256_of(path: &Path) -> Option<String> {
    let bytes = std::fs::read(path).ok()?;
    if bytes.is_empty() {
        return None;
    }
    Some(
        sha2::Sha256::digest(&bytes)
            .iter()
            .map(|byte| format!("{byte:02x}"))
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn agent_arguments_match_ce_shape() {
        let agent = AuthlibAgent {
            jar: PathBuf::from("/tmp/authlib-injector.jar"),
            server: "https://littleskin.cn/api/yggdrasil".into(),
            prefetched: "eyJtZXRhIjp7fX0=".into(),
        };
        let args = agent.arguments();
        assert_eq!(
            args[0],
            "-javaagent:/tmp/authlib-injector.jar=https://littleskin.cn/api/yggdrasil"
        );
        assert!(args.contains(&"-Dauthlibinjector.side=client".to_owned()));
        assert!(
            args.contains(&"-Dauthlibinjector.yggdrasil.prefetched=eyJtZXRhIjp7fX0=".to_owned())
        );
    }

    #[test]
    fn sha256_of_missing_files_is_none() {
        assert_eq!(
            sha256_of(Path::new("/nonexistent/authlib-injector.jar")),
            None
        );
    }
}
