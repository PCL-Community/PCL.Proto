//! IPC adapters grouped by business responsibility; command names stay stable.

use pcl_core::events::EventSink;

pub mod account;
pub mod config;
pub mod download;
pub mod export;
pub mod homepage;
pub mod installer;
pub mod instance_resources;
pub mod instances;
pub mod java;
pub mod launch;
pub mod launch_settings;
pub mod link;
pub mod logs;
pub mod memory;
pub mod network;
pub mod repositories;
pub mod servers;
pub mod session;
pub mod skin;
pub mod sources;
pub mod system;
pub mod versions;

/// 把 Tauri 的进度通道包装成核心库认识的普通回调；界面关闭后事件被丢弃。
pub(crate) fn channel_sink<T: serde::Serialize + Send + Sync + 'static>(
    channel: tauri::ipc::Channel<T>,
) -> EventSink<T> {
    pcl_core::events::event_sink(move |event| {
        let _ = channel.send(event);
    })
}
