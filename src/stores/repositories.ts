import type GameInstance from "@/types/gameInstance";
import { invoke } from "@tauri-apps/api/core";
import { defineStore } from "pinia";

export const useRepositoriesStore = defineStore('repositories', {
    state: () => ({
        repositires: [] as { name: string, path: string }[],
    }),
    actions: {
        async fetchFromBackend() {
            this.repositires = await invoke('get_repositories')
        },
        async getInstancesInRepository(repository_name: string): Promise<GameInstance[]> {
            return await invoke('get_instances_in_repository', { repository_name })
        }
    }
})
