import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import type GameInstance from '@/types/gameInstance'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { defineStore } from 'pinia'

type Repository = { name: string; path: string }

export const useRepositoriesStore = defineStore('repositories', {
  state: () => ({
    repositories: undefined as Repository[] | undefined,
    // shortcut: memory-only cache; persist it if cold-start flicker also matters.
    instancesByPath: {} as Record<string, GameInstance[]>,
  }),
  actions: {
    async fetchFromBackend() {
      this.repositories = await invoke<Repository[]>('get_repositories')
    },

    async selectRepository(index: number) {
      await invoke('select_repository', { repository_index: index })
    },

    async getInstancesInRepository(index: number): Promise<GameInstance[]> {
      return invoke<GameInstance[]>('get_instances_in_repository', { repository_index: index })
    },

    async addNew() {
      const modal = useModal()
      let file = await open({ multiple: false, directory: true })
      if (!file) {
        sideTip.show(pclText('Proto.Folder.NotSelected'), 'warn')
        return
      }
      let result = await modal.open({
        title: pclText('Proto.Folder.EnterName'),
        showInput: true,
        defaultInputText: file.split('/').pop(),
        buttons: [
          {
            content: pclText('Setup.Java.Add'),
            type: 'tint',
            operation() {
              if (modal.inputValue.value?.trim()) {
                modal.close(true)
              } else {
                sideTip.show(pclText('Proto.Folder.InvalidName'), 'warn')
              }
            },
          },
          {
            content: pclText('Common.Action.Cancel'),
            type: 'warn',
          },
        ],
      })
      if (result) {
        let repoUpdate = await invoke<Repository[]>('add_new_repository', {
          new_repo_path: file,
          new_repo_name: result.input,
        })
        this.repositories = repoUpdate
      }
    },
  },
})
