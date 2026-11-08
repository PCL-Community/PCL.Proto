//! Forge version metadata shared by the version list and installer.
use super::api_client::{McApiError, McApiResult, MinecraftApiClient};
use crate::io::net::http::resources::SourcePreference;
use serde_json::Value;

#[derive(Clone, Debug)]
pub(crate) struct ForgeVersion {
    pub version: String,
    pub file_version: String,
    pub category: String,
    pub hash: Option<String>,
    pub release_time: Option<String>,
}

impl ForgeVersion {
    pub fn installer_url(&self, game: &str) -> String {
        let extension = if self.category == "installer" {
            "jar"
        } else {
            "zip"
        };
        let id = format!("{}-{}", game.replace('-', "_"), self.file_version);
        format!(
            "https://files.minecraftforge.net/maven/net/minecraftforge/forge/{id}/forge-{id}-{}.{extension}",
            self.category
        )
    }

    fn matches(&self, version: &str) -> bool {
        self.version == version || self.file_version == version
    }
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value != "."
        && value != ".."
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'.' | b'-' | b'_' | b'+'))
}

fn checksum(value: Option<&str>) -> McApiResult<Option<String>> {
    let Some(hash) = value.filter(|hash| !hash.is_empty()) else {
        return Ok(None);
    };
    if !matches!(hash.len(), 32 | 40) || !hash.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(McApiError::InvalidMetadata(
            "Invalid Forge installer checksum".into(),
        ));
    }
    Ok(Some(hash.to_ascii_lowercase()))
}

fn mirror_versions(value: &Value, game: &str) -> McApiResult<Vec<ForgeVersion>> {
    let versions = value
        .as_array()
        .ok_or_else(|| McApiError::InvalidMetadata("Invalid Forge version list".into()))?;
    let mut result = Vec::new();
    for entry in versions {
        let Some(version) = entry
            .get("version")
            .and_then(Value::as_str)
            .filter(|v| token(v))
        else {
            continue;
        };
        let mut branch = entry
            .get("branch")
            .and_then(Value::as_str)
            .filter(|v| !v.is_empty());
        if matches!(version, "11.15.1.2318" | "11.15.1.1902" | "11.15.1.1890") {
            branch = Some("1.8.9");
        } else if branch.is_none()
            && game == "1.7.10"
            && version
                .split('.')
                .nth(3)
                .and_then(|v| v.parse::<u32>().ok())
                .is_some_and(|v| v >= 1300)
        {
            branch = Some("1.7.10");
        }
        if branch.is_some_and(|branch| !token(branch)) {
            continue;
        }
        let Some(files) = entry.get("files").and_then(Value::as_array) else {
            continue;
        };
        let chosen = [
            ("installer", "jar"),
            ("universal", "zip"),
            ("client", "zip"),
        ]
        .into_iter()
        .find_map(|(category, extension)| {
            files
                .iter()
                .find(|file| {
                    file["category"].as_str() == Some(category)
                        && file["format"].as_str() == Some(extension)
                })
                .map(|file| (category, file))
        });
        let Some((category, file)) = chosen else {
            continue;
        };
        let file_version = match branch {
            Some(branch) if !version.ends_with(&format!("-{branch}")) => {
                format!("{version}-{branch}")
            }
            _ => version.to_owned(),
        };
        result.push(ForgeVersion {
            version: version.to_owned(),
            file_version,
            category: category.into(),
            hash: checksum(file["hash"].as_str())?,
            release_time: entry
                .get("modified")
                .and_then(Value::as_str)
                .map(str::to_owned),
        });
    }
    Ok(result)
}

fn official_versions(html: &str, game: &str) -> McApiResult<Vec<ForgeVersion>> {
    let pattern = format!(
        r"forge-{}-(?P<version>[A-Za-z0-9._+\-]+)-(?P<category>installer|universal|client)\.(?P<ext>jar|zip)",
        regex::escape(&game.replace('-', "_"))
    );
    let files =
        regex::Regex::new(&pattern).map_err(|e| McApiError::InvalidMetadata(e.to_string()))?;
    let hash = regex::Regex::new(r"(?i)MD5:</strong>\s*([0-9a-f]{32})").unwrap();
    let timestamp = regex::Regex::new(r#"download-time" title="([^"]+)"#).unwrap();
    let mut result = Vec::new();
    for row in html.split("<td class=\"download-version").skip(1) {
        let mut chosen = None;
        for (category, extension) in [
            ("installer", "jar"),
            ("universal", "zip"),
            ("client", "zip"),
        ] {
            if let Some(file) = files
                .captures_iter(row)
                .find(|file| &file["category"] == category && &file["ext"] == extension)
            {
                chosen = Some(file);
                break;
            }
        }
        let Some(file) = chosen else { continue };
        let file_version = &file["version"];
        let version = file_version.split('-').next().unwrap_or(file_version);
        if !token(file_version) || !token(version) {
            continue;
        }
        let digest = hash
            .captures(&row[file.get(0).unwrap().end()..])
            .and_then(|v| v.get(1))
            .map(|v| v.as_str());
        let release_time = timestamp
            .captures(row)
            .and_then(|v| v.get(1))
            .map(|v| format!("{}Z", v.as_str().replace(' ', "T")));
        result.push(ForgeVersion {
            version: version.into(),
            file_version: file_version.into(),
            category: file["category"].into(),
            hash: checksum(digest)?,
            release_time,
        });
    }
    Ok(result)
}

fn sort_versions(versions: &mut [ForgeVersion]) {
    let numbers = |version: &str| {
        version
            .split('.')
            .map(|v| v.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    versions.sort_by(|a, b| {
        numbers(&b.version)
            .cmp(&numbers(&a.version))
            .then_with(|| b.file_version.cmp(&a.file_version))
    });
}

impl MinecraftApiClient {
    pub(crate) async fn forge_versions(
        &self,
        game: &str,
        selected: Option<&str>,
    ) -> McApiResult<Vec<ForgeVersion>> {
        if !token(game) || selected.is_some_and(|version| !token(version)) {
            return Err(McApiError::InvalidMetadata("Invalid Forge version".into()));
        }
        let mirror_first = !matches!(
            self.resources().settings().version_source,
            SourcePreference::Official
        );
        let mirror = format!(
            "https://bmclapi2.bangbang93.com/forge/minecraft/{}",
            game.replace('-', "_")
        );
        let official = format!(
            "https://files.minecraftforge.net/maven/net/minecraftforge/forge/index_{}.html",
            game.replace('-', "_")
        );
        let sources = if mirror_first {
            [(true, mirror), (false, official)]
        } else {
            [(false, official), (true, mirror)]
        };
        self.forge_versions_from_sources(game, selected, sources)
            .await
    }

    async fn forge_versions_from_sources(
        &self,
        game: &str,
        selected: Option<&str>,
        sources: [(bool, String); 2],
    ) -> McApiResult<Vec<ForgeVersion>> {
        let mut last_error = None;
        for (mirror, url) in sources {
            let result = if mirror {
                self.get::<Value>(&url, true)
                    .await
                    .and_then(|value| mirror_versions(&value, game))
            } else {
                self.get_bytes(&url).await.and_then(|bytes| {
                    let html = std::str::from_utf8(&bytes)
                        .map_err(|e| McApiError::InvalidMetadata(e.to_string()))?;
                    official_versions(html, game)
                })
            };
            match result {
                Ok(mut versions)
                    if !versions.is_empty()
                        && selected.is_none_or(|selected| {
                            versions.iter().any(|v| v.matches(selected))
                        }) =>
                {
                    sort_versions(&mut versions);
                    return Ok(versions);
                }
                Ok(_) => {
                    last_error = Some(McApiError::VersionNotFound(
                        selected.unwrap_or(game).to_owned(),
                    ))
                }
                Err(error) => last_error = Some(error),
            }
        }
        Err(last_error.unwrap_or_else(|| McApiError::VersionNotFound(game.into())))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_sources_select_supported_files_and_sort_versions() {
        let list = serde_json::json!([
            {"version":"10.13.4.9", "files":[{"category":"installer","format":"jar","hash":"c".repeat(40)}]},
            {"version":"10.13.4.1614", "branch":null, "modified":"2020-01-01T00:00:00Z",
             "files":[{"category":"universal","format":"zip","hash":"a".repeat(32)},
                      {"category":"installer","format":"jar","hash":"b".repeat(40)}]},
            {"version":"10.13.4.99", "files":[{"category":"mdk","format":"zip","hash":"d".repeat(40)}]}
        ]);
        let mut versions = mirror_versions(&list, "1.7.10").unwrap();
        sort_versions(&mut versions);
        assert_eq!(versions.len(), 2);
        let file = &versions[0];
        assert_eq!(file.version, "10.13.4.1614");
        assert!(
            file.installer_url("1.7.10")
                .ends_with("forge-1.7.10-10.13.4.1614-1.7.10-installer.jar")
        );
        assert_eq!(
            file.hash.as_deref(),
            Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );
        assert_eq!(file.release_time.as_deref(), Some("2020-01-01T00:00:00Z"));

        let html = r#"<td class="download-version">10.13.4.1614
            <td class="download-time" title="2021-02-15 03:24:02">
            forge-1.7.10-10.13.4.1614-1.7.10-universal.zip MD5:</strong> aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
            forge-1.7.10-10.13.4.1614-1.7.10-installer.jar MD5:</strong> bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"#;
        let versions = official_versions(html, "1.7.10").unwrap();
        assert_eq!(versions.len(), 1);
        let file = &versions[0];
        assert_eq!(file.version, "10.13.4.1614");
        assert_eq!(file.file_version, "10.13.4.1614-1.7.10");
        assert_eq!(
            file.hash.as_deref(),
            Some("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb")
        );
        assert_eq!(file.release_time.as_deref(), Some("2021-02-15T03:24:02Z"));
        assert!(!token("../escape"));
    }

    #[tokio::test]
    async fn version_sources_fall_back_after_failure_or_missing_selection() {
        use super::super::api_client::ApiProvider;
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            let mut requests = Vec::new();
            for _ in 0..4 {
                let (mut stream, _) = listener.accept().await.unwrap();
                let mut input = [0; 4096];
                let count = stream.read(&mut input).await.unwrap();
                let request = String::from_utf8_lossy(&input[..count]);
                let path = request.split_whitespace().nth(1).unwrap();
                requests.push(path.to_owned());
                let (status, body) = match path {
                    "/bad" => ("503 Service Unavailable", "unavailable"),
                    "/absent" => (
                        "200 OK",
                        r#"[{"version":"47.1.0","files":[{"category":"installer","format":"jar","hash":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"}]}]"#,
                    ),
                    "/official" => (
                        "200 OK",
                        r#"<td class="download-version">47.0.1 forge-1.20.1-47.0.1-installer.jar MD5:</strong> bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb"#,
                    ),
                    _ => panic!("Unexpected path {path}"),
                };
                stream.write_all(format!("HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).as_bytes()).await.unwrap();
            }
            requests
        });
        let client = MinecraftApiClient::new(
            crate::io::net::http::HttpTransport::default(),
            &ApiProvider::Official,
        );
        let url = |path| format!("http://{address}/{path}");
        for first in ["bad", "absent"] {
            let versions = client
                .forge_versions_from_sources(
                    "1.20.1",
                    Some("47.0.1"),
                    [(true, url(first)), (false, url("official"))],
                )
                .await
                .unwrap();
            assert_eq!(versions[0].version, "47.0.1");
        }
        assert_eq!(
            server.await.unwrap(),
            ["/bad", "/official", "/absent", "/official"]
        );
    }

    #[tokio::test]
    #[ignore = "Reads current Forge version lists from the official and BMCL sources"]
    async fn real_forge_sources_expose_the_same_installer() {
        use super::super::api_client::ApiProvider;
        let client = MinecraftApiClient::new(
            crate::io::net::http::HttpTransport::default(),
            &ApiProvider::Official,
        );
        let mirror: Value = client
            .get(
                "https://bmclapi2.bangbang93.com/forge/minecraft/1.20.1",
                false,
            )
            .await
            .unwrap();
        let html = if let Some(path) = std::env::var_os("PCL_FORGE_OFFICIAL_HTML") {
            std::fs::read(path).unwrap()
        } else {
            client
                .get_bytes(
                    "https://files.minecraftforge.net/maven/net/minecraftforge/forge/index_1.20.1.html",
                )
                .await
                .unwrap()
        };
        let mirror = mirror_versions(&mirror, "1.20.1").unwrap();
        let official = official_versions(std::str::from_utf8(&html).unwrap(), "1.20.1").unwrap();
        for versions in [mirror, official] {
            let entry = versions
                .iter()
                .find(|entry| entry.version == "47.0.1")
                .unwrap();
            assert_eq!(entry.category, "installer");
            assert!(entry.hash.is_some());
            assert!(
                entry
                    .installer_url("1.20.1")
                    .ends_with("forge-1.20.1-47.0.1-installer.jar")
            );
        }
    }
}
