//! Shared per-client HTTP quotas, byte accounting and bandwidth budget.
use std::{
    sync::{
        Arc, Mutex, RwLock,
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};
use tokio::sync::Notify;

#[derive(Debug, Clone, Copy, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum SourcePreference {
    Mirror = 0,
    #[default]
    Auto = 1,
    Official = 2,
}

/// 请求类型：不同类型的下载各自在设置里有独立的源偏好。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    /// 资源文件：游戏文件、Mod 文件、整合包等。
    File,
    /// 版本元数据：版本清单及其引用到的 JSON。
    VersionMetadata,
}

#[derive(Debug, Clone, Copy, Default, serde_repr::Serialize_repr, serde_repr::Deserialize_repr)]
#[repr(u8)]
pub enum HttpMode {
    #[default]
    Auto = 0,
    Http11 = 1,
    Http2 = 2,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(default)]
pub struct DownloadOptions {
    pub max_connections: usize,
    pub file_connections: usize,
    pub speed_limit: u64,
    pub http_mode: HttpMode,
    pub file_source: SourcePreference,
    pub version_source: SourcePreference,
    /// 社区资源（CurseForge / Modrinth）的 API 与文件；`Mirror` 表示优先 MCIM。
    pub community_source: SourcePreference,
}
impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            max_connections: 64,
            file_connections: 8,
            speed_limit: 0,
            http_mode: HttpMode::Auto,
            file_source: SourcePreference::Auto,
            version_source: SourcePreference::Auto,
            community_source: SourcePreference::Auto,
        }
    }
}
impl DownloadOptions {
    /// 某类请求实际生效的源偏好。
    pub fn preference(&self, kind: SourceKind) -> SourcePreference {
        match kind {
            SourceKind::File => self.file_source,
            SourceKind::VersionMetadata => self.version_source,
        }
    }
    pub fn validate(&self) -> Result<(), String> {
        if !(1..=256).contains(&self.max_connections) || !(1..=8).contains(&self.file_connections) {
            return Err("Invalid download connection count or speed limit".into());
        }
        Ok(())
    }
}

#[derive(Debug, serde::Serialize)]
pub struct DownloadStats {
    pub active_connections: usize,
    pub max_connections: usize,
    pub speed: f64,
    pub downloaded_bytes: u64,
}
struct SpeedSample {
    at: Instant,
    bytes: u64,
    speed: f64,
}
struct Bandwidth {
    at: Instant,
    tokens: f64,
}

pub struct DownloadResources {
    settings: RwLock<DownloadOptions>,
    active: AtomicUsize,
    bytes: AtomicU64,
    speed: Mutex<SpeedSample>,
    bandwidth: tokio::sync::Mutex<Bandwidth>,
    changed: Notify,
    pub official_fast: AtomicBool,
}

impl Default for DownloadResources {
    fn default() -> Self {
        Self {
            settings: RwLock::new(DownloadOptions::default()),
            active: AtomicUsize::new(0),
            bytes: AtomicU64::new(0),
            speed: Mutex::new(SpeedSample {
                at: Instant::now(),
                bytes: 0,
                speed: 0.0,
            }),
            bandwidth: tokio::sync::Mutex::new(Bandwidth {
                at: Instant::now(),
                tokens: 128.0 * 1024.0,
            }),
            changed: Notify::new(),
            official_fast: AtomicBool::new(true),
        }
    }
}
impl DownloadResources {
    pub fn settings(&self) -> DownloadOptions {
        self.settings.read().unwrap().clone()
    }
    pub fn configure(&self, settings: DownloadOptions) -> Result<(), String> {
        settings.validate()?;
        *self.settings.write().unwrap() = settings;
        self.changed.notify_waiters();
        Ok(())
    }
    pub async fn acquire(self: &Arc<Self>) -> ConnectionLease {
        loop {
            let changed = self.changed.notified();
            tokio::pin!(changed);
            changed.as_mut().enable();
            let active = self.active.load(Ordering::SeqCst);
            if active < self.settings().max_connections
                && self
                    .active
                    .compare_exchange(active, active + 1, Ordering::SeqCst, Ordering::SeqCst)
                    .is_ok()
            {
                return ConnectionLease(self.clone());
            }
            // A failed CAS may have raced an acquisition rather than a release.
            if self.active.load(Ordering::SeqCst) < self.settings().max_connections {
                continue;
            }
            changed.await;
        }
    }
    pub fn record_bytes(&self, count: usize) {
        self.bytes.fetch_add(count as u64, Ordering::Relaxed);
    }
    pub fn stats(&self) -> DownloadStats {
        let bytes = self.bytes.load(Ordering::Relaxed);
        let active = self.active.load(Ordering::SeqCst);
        let mut sample = self.speed.lock().unwrap();
        let elapsed = sample.at.elapsed().as_secs_f64();
        if elapsed >= 0.25 || active == 0 {
            sample.speed = if active == 0 {
                0.0
            } else {
                bytes.saturating_sub(sample.bytes) as f64 / elapsed.max(0.001)
            };
            sample.at = Instant::now();
            sample.bytes = bytes;
        }
        DownloadStats {
            active_connections: active,
            max_connections: self.settings().max_connections,
            speed: sample.speed,
            downloaded_bytes: bytes,
        }
    }
    pub async fn throttle(&self, mut bytes: usize) {
        while bytes > 0 {
            let amount = bytes.min(64 * 1024);
            loop {
                let changed = self.changed.notified();
                tokio::pin!(changed);
                changed.as_mut().enable();
                let limit = self.settings().speed_limit;
                let mut budget = self.bandwidth.lock().await;
                let elapsed = budget.at.elapsed().as_secs_f64();
                budget.at = Instant::now();
                budget.tokens = (budget.tokens + elapsed * limit as f64).min(128.0 * 1024.0);
                if limit == 0 || budget.tokens >= amount as f64 {
                    if limit > 0 {
                        budget.tokens -= amount as f64;
                    }
                    break;
                }
                let delay = Duration::from_secs_f64(
                    ((amount as f64 - budget.tokens) / limit as f64).min(2.0),
                );
                drop(budget);
                tokio::select! { () = tokio::time::sleep(delay) => {}, () = &mut changed => {} }
            }
            bytes -= amount;
        }
    }
}
pub struct ConnectionLease(Arc<DownloadResources>);
impl Drop for ConnectionLease {
    fn drop(&mut self) {
        self.0.active.fetch_sub(1, Ordering::SeqCst);
        self.0.changed.notify_waiters();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn bandwidth_budget_is_shared_across_concurrent_transfers() {
        let resources = Arc::new(DownloadResources::default());
        resources
            .configure(DownloadOptions {
                speed_limit: 1024 * 1024,
                ..Default::default()
            })
            .unwrap();
        let started = Instant::now();
        tokio::join!(
            resources.throttle(256 * 1024),
            resources.throttle(256 * 1024)
        );
        assert!(started.elapsed() >= Duration::from_millis(350));
    }
    #[tokio::test]
    async fn global_quota_is_shared_live_and_cancellation_safe() {
        let resources = Arc::new(DownloadResources::default());
        resources
            .configure(DownloadOptions {
                max_connections: 1,
                ..Default::default()
            })
            .unwrap();
        let first = resources.acquire().await;
        assert_eq!(resources.stats().active_connections, 1);
        assert!(
            tokio::time::timeout(Duration::from_millis(10), resources.acquire())
                .await
                .is_err()
        );
        resources
            .configure(DownloadOptions {
                max_connections: 2,
                ..Default::default()
            })
            .unwrap();
        let second = resources.acquire().await;
        resources.record_bytes(7);
        assert_eq!(resources.stats().downloaded_bytes, 7);
        resources
            .configure(DownloadOptions {
                max_connections: 1,
                ..Default::default()
            })
            .unwrap();
        drop(first);
        assert!(
            tokio::time::timeout(Duration::from_millis(10), resources.acquire())
                .await
                .is_err()
        );
        drop(second);
        drop(
            tokio::time::timeout(Duration::from_secs(1), resources.acquire())
                .await
                .unwrap(),
        );
        assert_eq!(resources.stats().active_connections, 0);
        assert_eq!(resources.stats().speed, 0.0);
        assert!(
            resources
                .configure(DownloadOptions {
                    max_connections: 257,
                    ..Default::default()
                })
                .is_err()
        );
    }
    #[tokio::test]
    async fn bandwidth_wait_is_cancel_safe_and_can_be_disabled_live() {
        let resources = Arc::new(DownloadResources::default());
        resources
            .configure(DownloadOptions {
                speed_limit: 1,
                ..Default::default()
            })
            .unwrap();
        resources.throttle(128 * 1024).await;
        assert!(
            tokio::time::timeout(Duration::from_millis(10), resources.throttle(1))
                .await
                .is_err()
        );
        resources.configure(DownloadOptions::default()).unwrap();
        tokio::time::timeout(Duration::from_secs(1), resources.throttle(1024))
            .await
            .unwrap();
    }
}
