use std::{
    collections::HashMap,
    future::Future,
    sync::{
        Arc, LazyLock, Mutex,
        atomic::{AtomicBool, Ordering},
    },
};
use tokio::sync::{Mutex as AsyncMutex, watch};

struct Task {
    cancelled: watch::Sender<bool>,
    blocking: Arc<AsyncMutex<()>>,
    committed: AtomicBool,
}

static TASKS: LazyLock<Mutex<HashMap<i32, Arc<Task>>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[tauri::command(rename_all = "snake_case")]
pub fn prepare_download_task(task_id: i32) -> Result<(), String> {
    let mut tasks = TASKS.lock().map_err(|e| e.to_string())?;
    if tasks.contains_key(&task_id) {
        return Err("Download task already exists".into());
    }
    tasks.insert(
        task_id,
        Arc::new(Task {
            cancelled: watch::channel(false).0,
            blocking: Arc::new(AsyncMutex::new(())),
            committed: AtomicBool::new(false),
        }),
    );
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn cancel_download_task(task_id: i32) -> Result<(), String> {
    let tasks = TASKS.lock().map_err(|e| e.to_string())?;
    if let Some(task) = tasks.get(&task_id) {
        task.cancelled.send_replace(true);
    }
    Ok(())
}

#[tauri::command(rename_all = "snake_case")]
pub fn release_download_task(task_id: i32) {
    if let Ok(mut tasks) = TASKS.lock() {
        tasks.remove(&task_id);
    }
}

pub async fn run<T>(
    task_id: i32,
    future: impl Future<Output = Result<T, String>>,
) -> Result<T, String> {
    let task = {
        let mut tasks = TASKS.lock().map_err(|e| e.to_string())?;
        tasks
            .entry(task_id)
            .or_insert_with(|| {
                Arc::new(Task {
                    cancelled: watch::channel(false).0,
                    blocking: Arc::new(AsyncMutex::new(())),
                    committed: AtomicBool::new(false),
                })
            })
            .clone()
    };
    struct Remove(i32, Arc<Task>);
    impl Drop for Remove {
        fn drop(&mut self) {
            if let Ok(mut tasks) = TASKS.lock() {
                if tasks
                    .get(&self.0)
                    .is_some_and(|task| Arc::ptr_eq(task, &self.1))
                {
                    tasks.remove(&self.0);
                }
            }
        }
    }
    let _remove = Remove(task_id, task.clone());
    let mut cancelled = task.cancelled.subscribe();
    tokio::pin!(future);
    let result = tokio::select! {
        biased;
        _ = async { if !*cancelled.borrow() { let _ = cancelled.changed().await; } } => {
            let _guard = task.blocking.lock().await;
            if task.committed.load(Ordering::Acquire) {
                future.await
            } else {
                Err("Download cancelled".into())
            }
        }
        result = &mut future => result,
    };
    result
}

pub async fn commit<T: Send + 'static>(
    task_id: i32,
    operation: impl FnOnce() -> anyhow::Result<T> + Send + 'static,
) -> anyhow::Result<T> {
    let task = TASKS
        .lock()
        .ok()
        .and_then(|tasks| tasks.get(&task_id).cloned());
    let guard = match &task {
        Some(task) => Some(task.blocking.clone().lock_owned().await),
        None => None,
    };
    tokio::task::spawn_blocking(move || {
        let _guard = guard;
        let result = operation();
        if result.is_ok() {
            if let Some(task) = task {
                task.committed.store(true, Ordering::Release);
            }
        }
        result
    })
    .await?
}

pub async fn blocking<T: Send + 'static>(
    task_id: i32,
    operation: impl FnOnce() -> T + Send + 'static,
) -> Result<T, tokio::task::JoinError> {
    let task = TASKS
        .lock()
        .ok()
        .and_then(|tasks| tasks.get(&task_id).cloned());
    let guard = match task {
        Some(task) => Some(task.blocking.clone().lock_owned().await),
        None => None,
    };
    tokio::task::spawn_blocking(move || {
        let _guard = guard;
        operation()
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::AtomicBool;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn cancellation_before_start_does_not_run_future() {
        let id = -1001;
        prepare_download_task(id).unwrap();
        cancel_download_task(id).unwrap();
        let ran = AtomicBool::new(false);
        let result = run(id, async {
            ran.store(true, Ordering::Relaxed);
            Ok(())
        })
        .await;
        assert_eq!(result, Err("Download cancelled".into()));
        assert!(!ran.load(Ordering::Relaxed));
    }

    #[tokio::test]
    async fn cancellation_waits_for_blocking_work_and_preserves_successful_commit() {
        for (id, commit_work) in [(-1002, false), (-1003, true)] {
            prepare_download_task(id).unwrap();
            let (started_tx, started_rx) = oneshot::channel();
            let (finish_tx, finish_rx) = std::sync::mpsc::channel();
            let job = tokio::spawn(run(id, async move {
                let work = move || {
                    started_tx.send(()).unwrap();
                    finish_rx.recv().unwrap();
                    Ok::<_, anyhow::Error>(())
                };
                if commit_work {
                    commit(id, work).await.map_err(|e| e.to_string())?
                } else {
                    blocking(id, work)
                        .await
                        .map_err(|e| e.to_string())?
                        .map_err(|e| e.to_string())?
                }
                Ok(())
            }));
            started_rx.await.unwrap();
            cancel_download_task(id).unwrap();
            finish_tx.send(()).unwrap();
            let result = job.await.unwrap();
            if commit_work {
                assert_eq!(result, Ok(()));
            } else {
                assert_eq!(result, Err("Download cancelled".into()));
            }
        }
    }
}
