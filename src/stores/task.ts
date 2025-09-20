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

type TaskEvent =
    | {
        event: 'update_item', data: {
            item_id: number,
            task_id: number,
            files_remaining: number,
            overall_progress: number
        }
    }
    | {
        event: 'created', data: {
            id: number,
            task_items: ITaskItem[]
        }
    }

export const useTaskManager = defineStore('task-manager', () => {
    const tasks = ref<ITask[]>([])
    const totalProgress = computed(() => 0)
    const totalSpeed = computed(() => 0)
    const totalRemaining = computed(() => {
        return tasks.value.reduce((total, task) => {
            return total + task.items.reduce((sum, item) => sum + item.remaining, 0)
        }, 0);
    })
    const onEvent = new Channel()
    onEvent.onmessage = (message) => {
        console.log('got task event', message)
    }
    function StartDownloadMCVersion(version_id: string) {
        invoke('download_minecraft_version', { on_event: onEvent })
    }
    return {
        tasks,
        totalProgress,
        totalSpeed,
        totalRemaining,
        StartDownloadMCVersion
    }
})