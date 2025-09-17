import { defineStore } from "pinia";

enum TaskStatus {
    Pending,
    Running,
    Completed,
    Failed,
}

export interface ITask {
    id: string;
    name: string;
    status: TaskStatus;
    speed: number;
    items: ITaskItem[];
    progress: number;
}

export interface ITaskItem {
    id: string;
    name: string;
    status: TaskStatus;
    speed: number;
    progress: number;
}

export const useTaskManager = defineStore('task', {
    state: () => ({
        tasks: [] as ITask[],
    }),
})