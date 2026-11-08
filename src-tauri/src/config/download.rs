use pcl_core::io::net::http::resources::{HttpMode, SourcePreference};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct DownloadSettings {
    pub max_connections: usize,
    pub file_connections: usize,
    pub speed_limit: u64,
    pub http_mode: HttpMode,
    pub file_source: SourcePreference,
    pub version_source: SourcePreference,
    /// 社区资源（CurseForge / Modrinth）的 API 与文件来源。
    pub community_source: SourcePreference,
    pub auto_select_instance: Option<bool>,
}
impl Default for DownloadSettings {
    fn default() -> Self {
        Self {
            max_connections: 64,
            file_connections: 8,
            speed_limit: 0,
            http_mode: HttpMode::Auto,
            file_source: SourcePreference::Auto,
            version_source: SourcePreference::Auto,
            community_source: SourcePreference::Auto,
            auto_select_instance: None,
        }
    }
}
impl DownloadSettings {
    pub fn options(&self) -> pcl_core::io::net::http::resources::DownloadOptions {
        pcl_core::io::net::http::resources::DownloadOptions {
            max_connections: self.max_connections,
            file_connections: self.file_connections,
            speed_limit: self.speed_limit,
            http_mode: self.http_mode,
            file_source: self.file_source,
            version_source: self.version_source,
            community_source: self.community_source,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if self.speed_limit > 20 * 1024 * 1024 {
            return Err("Invalid download speed limit".into());
        }
        self.options().validate()
    }
}
