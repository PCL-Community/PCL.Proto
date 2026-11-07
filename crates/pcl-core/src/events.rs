//! 后台任务向宿主界面回传进度。
//!
//! 核心库不认识 Tauri 的 `Channel`，只接受普通回调；宿主把回调接到自己的 IPC 通道上。

use std::sync::Arc;

/// 事件接收器；`Send + Sync` 以便在并发任务间共享。
pub type EventSink<T> = Arc<dyn Fn(T) + Send + Sync>;

/// 把普通闭包包装成 [`EventSink`]。
pub fn event_sink<T, F>(handler: F) -> EventSink<T>
where
    F: Fn(T) + Send + Sync + 'static,
{
    Arc::new(handler)
}
