//! PCL.Proto 联机服务层：公告、Natayark 账户、大厅状态与 Terracotta 桥接。

pub mod announcement;
pub mod naid;
pub mod service;

use std::sync::LazyLock;

/// 公告协议版本；服务端下发更高版本时要求更新启动器。
pub const LOBBY_PROTOCOL_VERSION: u32 = 6;

/// 启动器版本代号，用于公告的 minVer/maxVer 过滤。
pub static LAUNCHER_VERSION_CODE: LazyLock<u32> = LazyLock::new(|| {
    let mut parts = env!("CARGO_PKG_VERSION")
        .split('.')
        .map(|part| part.parse::<u32>().unwrap_or_default());
    let major = parts.next().unwrap_or_default();
    let minor = parts.next().unwrap_or_default();
    let patch = parts.next().unwrap_or_default();
    major * 10_000 + minor * 100 + patch
});

/// IPC 错误：前端按 code 映射到本地化文案。
#[derive(Clone, Debug, serde::Serialize)]
pub struct LinkError {
    pub code: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

impl LinkError {
    pub fn new(code: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            args: Vec::new(),
        }
    }
}

impl std::fmt::Display for LinkError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}", self.code)
    }
}

impl std::error::Error for LinkError {}

impl From<String> for LinkError {
    fn from(error: String) -> Self {
        if matches!(
            error.as_str(),
            "invalid_username" | "invalid_code" | "join_failed"
        ) {
            Self::new(error)
        } else {
            log::warn!("Link configuration update failed: {error}");
            Self::new("config_failed")
        }
    }
}
