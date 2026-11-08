//! 联机（Link）设置与持久状态；对照 PCL CE 的 Config.Link / States.Link。

use serde::{Deserialize, Serialize};

/// 用户可修改的联机设置；网络选项直接复用核心类型。
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct LinkSettings {
    /// 空表示使用 Natayark 用户名。
    pub username: String,
    #[serde(flatten)]
    pub options: pcl_core::link::LinkOptions,
}

impl LinkSettings {
    pub fn validate(&self) -> Result<(), String> {
        let username = self.username.trim();
        if username.contains('|')
            || username.chars().any(char::is_control)
            || username.chars().count() > 64
        {
            return Err("invalid_username".into());
        }
        Ok(())
    }
}

/// 不属于用户设置的持久状态：EULA、公告缓存与 Natayark 登录令牌。
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct LinkState {
    pub eula_accepted: bool,
    pub announce_cache: Option<String>,
    pub announce_cache_version: u64,
    pub naid_refresh_token: Option<String>,
    pub naid_refresh_expires_at: Option<String>,
}

impl LinkState {
    pub fn clear_credentials(&mut self) {
        self.naid_refresh_token = None;
        self.naid_refresh_expires_at = None;
    }

    pub fn clear_announcement_cache(&mut self) {
        self.announce_cache = None;
        self.announce_cache_version = 0;
    }
}

/// 沿用旧 Terracotta 的 16 字节身份，同时分离启动器自己的标识文件。
pub fn machine_id_path(config_dir: &std::path::Path) -> std::path::PathBuf {
    use std::io::Write;
    let path = config_dir.join("link_machine_id.bin");
    if !path.exists()
        && let Ok(bytes) = std::fs::read(config_dir.join("pcl_identifier.txt"))
        && bytes.len() == 16
    {
        let mut options = std::fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        if let Err(error) = options
            .open(&path)
            .and_then(|mut file| file.write_all(&bytes))
        {
            log::warn!("Cannot migrate Link machine identity: {error}");
        }
    }
    path
}
