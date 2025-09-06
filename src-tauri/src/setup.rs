pub mod constants {
    pub const DEFAULT_JAVA_LIST_CACHE_VERSION: i32 = 0;
    pub const LAUNCHER_NAME: &str = "PCL.Proto";
    pub const USER_AGENT: &str = "PCL-Community/PCL.Proto/0.5.0";
}

/// PCL global setup info
pub struct PCLSetupInfo {
    java_list_cache_version: i32,
    theme: Theme,
    download_sourse: DownloadSource,
}

/// PCL theme
enum Theme {
    BlueLight,
    BlueDark,
}

enum DownloadSource {
    Official,
    BMCLApi,
}

/// hard encoded default setup info
const DEFAULT_SETUP_INFO: PCLSetupInfo = PCLSetupInfo {
    java_list_cache_version: constants::DEFAULT_JAVA_LIST_CACHE_VERSION,
    theme: Theme::BlueLight,
    download_sourse: DownloadSource::Official,
};

/// PCL setup info default impl
impl Default for PCLSetupInfo {
    /// PCL setup info default impl
    fn default() -> Self {
        // TODO: 应该先从缓存中读取
        DEFAULT_SETUP_INFO
    }
}
