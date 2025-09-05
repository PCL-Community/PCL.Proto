pub mod constants {
    pub const DEFAULT_JAVA_LIST_CACHE_VERSION: i32 = 0;
    pub const LAUNCHER_NAME: &str = "PCL.Proto";
    pub const USER_AGENT: &str = "PCL-Community/PCL.Proto/0.5.0";
}

/// PCL 全局配置
pub struct PCLSetupInfo {
    java_list_cache_version: i32,
    theme: Theme,
    download_sourse: DownloadSource,
}

/// 主题
enum Theme {
    BlueLight,
    BlueDark,
}

enum DownloadSource {
    Official,
    BMCLApi,
}

/// 硬编码的可变默认配置
const DEFAULT_SETUP_INFO: PCLSetupInfo = PCLSetupInfo {
    java_list_cache_version: constants::DEFAULT_JAVA_LIST_CACHE_VERSION,
    theme: Theme::BlueLight,
    download_sourse: DownloadSource::Official,
};

/// 启动时从缓存中读取当前配置
impl Default for PCLSetupInfo {
    fn default() -> Self {
        // TODO: 应该先从缓存中读取
        DEFAULT_SETUP_INFO
    }
}
