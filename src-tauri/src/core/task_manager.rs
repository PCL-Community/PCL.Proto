use std::sync::LazyLock;

#[derive(Debug)]
pub enum TaskStatus {
    Pending,
    Running,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub status: TaskStatus,
    pub items: Vec<TaskItem>,
    pub speed: f64,
    pub progress: f64,
}

#[derive(Debug)]
pub struct TaskItem {
    pub name: String,
    pub progress: f64,
}

#[derive(Default)]
pub struct TaskManager {
    pub tasks: Vec<Task>,
}

static TASK_MANAGER: LazyLock<TaskManager> = LazyLock::new(|| TaskManager::default());

impl TaskManager {
    pub fn instance() -> &'static TaskManager {
        &TASK_MANAGER
    }
}
