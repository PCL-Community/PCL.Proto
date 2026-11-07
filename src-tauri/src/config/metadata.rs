//! Launcher metadata constants, mirroring `PCL.Core.App.Metadata`.

pub mod constants {
    pub const DEFAULT_JAVA_LIST_CACHE_VERSION: i32 = 0;
    pub const LAUNCHER_NAME: &str = "PCL.Proto";
    pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
    pub const USER_AGENT: &str = concat!("PCL-Community/PCL.Proto/", env!("CARGO_PKG_VERSION"));
}

/// 全局版本说明的默认值，对照 CE 的品牌值。
pub fn default_version_info() -> String {
    constants::LAUNCHER_NAME.to_owned()
}
