import { defineStore } from "pinia";

type TaskStatus = 'pending' | 'running' | 'success' | 'failed';

export interface ITask {
    id: string;
    name: string;
    status: TaskStatus;
    speed: number;
    items: ITaskItem[];
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