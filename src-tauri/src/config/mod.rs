//! 启动器配置：文件位置、读写、内存状态、策略与档案持久化。
//!
//! 在这里解析默认值、继承与用户偏好，再向 `pcl-core` 传递具体操作参数。

pub mod arguments;
pub mod download;
pub mod instance;
pub mod isolation;
pub mod java;
pub mod java_preference;
pub mod launch;
pub mod launch_settings;
pub mod link;
pub mod manager;
pub mod memory;
pub mod metadata;
pub mod network;
pub mod paths;
pub mod profiles;
pub mod states;
pub mod system;

pub use java::active_instance as current_instance;
pub use states::WindowSettings;

/// serde_yaml 会把 UTF-8 BOM 当成第二个空文档而拒绝整个文件，CE 写出的配置常带 BOM。
pub(crate) fn strip_bom(bytes: &[u8]) -> &[u8] {
    bytes.strip_prefix(b"\xEF\xBB\xBF").unwrap_or(bytes)
}
