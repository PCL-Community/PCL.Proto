import type GameInstance from '@/types/gameInstance'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { defineStore } from 'pinia'

export const useRepositoriesStore = defineStore('repositories', {
  state: () => ({
    repositires: [] as { name: string; path: string }[],
  }),
  actions: {
    async fetchFromBackend() {
      this.repositires = await invoke('get_repositories')
    },
    async getInstancesInRepository(index: number): Promise<GameInstance[]> {
      return await invoke('get_instances_in_repository', { repository_index: index })
    },
    async addNew() {
      const repo = await open({ multiple: false, directory: true })
      console.log(repo)
    },
  },
})
