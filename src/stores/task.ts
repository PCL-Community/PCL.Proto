import { Channel, invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

export class Task {
    id: number;
    name: string;
    status: TaskStatus;
    speed: number;
    items: TaskItem[];
    progress: number;

    constructor(id: number, version_id: string) {
        this.id = id;
        this.name = version_id + "下载";
        this.status = TaskStatus.Pending;
        this.speed = 0;
        this.items = [
            new TaskItem(1, "json download", id, 1),
            new TaskItem(2, "version jar", id, 1),
            new TaskItem(3, "libraries", id),
        ]
        this.progress = 0
    }
}

export class TaskItem {
    id: number;
    task_id: number;
    name: string;
    status: TaskStatus;
    speed: number;
    progress: number;
    remaining: number | undefined;

    constructor(id: number, name: string, task_id: number, files_num?: number) {
        this.id = id;
        this.name = name;
        this.task_id = task_id;
        this.status = TaskStatus.Pending;
        this.progress = 0;
        this.remaining = files_num;
        this.speed = 0;
    }
}

interface TaskEvent {
    item_id: number,
    task_id: number,
    files_remaining: number,
    overall_progress: number,
    status: TaskStatus
}


export const useTaskManager = defineStore('task-manager', () => {
    const tasks = ref<Task[]>([])
    let current_taskid = -1
    const totalProgress = computed(() => 0)
    const totalSpeed = computed(() => 0)
    const totalRemaining = computed(() => {
        return tasks.value.reduce((total, task) => {
            return total + task.items.reduce((sum, item) => sum + (item.remaining || 0), 0)
        }, 0);
    })

    async function startDownloadMCVersion(version_id: string) {
        let newTask = new Task(++current_taskid, version_id)
        tasks.value.push(newTask)
        const onEvent = new Channel<TaskEvent>()
        onEvent.onmessage = (message) => {
            // console.log('got task event', message)
            switch (message.status) {
                case TaskStatus.Running:
                    break;
                case TaskStatus.Pending:
                    break;
            }
        }
        await invoke('download_minecraft_version', { on_event: onEvent, version_id, task_id: current_taskid })
    }

    return {
        tasks,
        totalProgress,
        totalSpeed,
        totalRemaining,
        startDownloadMCVersion
    }
})