import { Channel, invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";
import { computed, ref } from "vue";

enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

export interface ITask {
    id: number;
    name: string;
    status: TaskStatus;
    speed: number;
    items: ITaskItem[];
    progress: number;
}

export interface ITaskItem {
    id: number;
    task_id: number,
    name: string;
    status: TaskStatus;
    speed: number;
    progress: number;
    remaining: number;
}

interface TaskEvent {
    item_id: number,
    task_id: number,
    files_remaining: number,
    overall_progress: number,
    status: TaskStatus
}


export const useTaskManager = defineStore('task-manager', () => {
    const tasks = ref<ITask[]>([])
    let max_task_id = -1
    const totalProgress = computed(() => 0)
    const totalSpeed = computed(() => 0)
    const totalRemaining = computed(() => {
        return tasks.value.reduce((total, task) => {
            return total + task.items.reduce((sum, item) => sum + item.remaining, 0)
        }, 0);
    })
    const onEvent = new Channel<TaskEvent>()
    onEvent.onmessage = (message) => {
        console.log('got task event', message)
        switch (message.status) {
            case TaskStatus.Running:
                break;
            case TaskStatus.Pending:
                break;
        }
    }
    function StartDownloadMCVersion(version_id: string) {
        let newTask: ITask = {
            id: max_task_id++,
            name: version_id + "下载",
            progress: 0,
            speed: 0,
            status: TaskStatus.Pending,
            items: [{ id: 0, name: "下载json", progress: 0, remaining: 1, speed: 0, status: TaskStatus.Pending, task_id: max_task_id }]
        }
        tasks.value.push(newTask)
        invoke('download_minecraft_version', { on_event: onEvent, version_id })
    }
    return {
        tasks,
        totalProgress,
        totalSpeed,
        totalRemaining,
        StartDownloadMCVersion
    }
})