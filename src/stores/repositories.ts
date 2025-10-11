import sideTip from '@/composables/sideTip'
import { useModal } from '@/composables/useModal'
import type GameInstance from '@/types/gameInstance'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { defineStore } from 'pinia'

type Repository = { name: string; path: string }

export const useRepositoriesStore = defineStore('repositories', {
  state: () => ({
    repositires: undefined as Repository[] | undefined,
  }),
  actions: {
    async fetchFromBackend() {
      this.repositires = await invoke<Repository[]>('get_repositories')
    },

    async getInstancesInRepository(index: number): Promise<GameInstance[]> {
      return invoke<GameInstance[]>('get_instances_in_repository', { repository_index: index })
    },

    async addNew() {
      const modal = useModal()
      let file = await open({ multiple: false, directory: true })
      if (!file) {
        sideTip.show('未选择任何文件夹！', 'warn')
        return
      }
      let result = await modal.open({
        title: '输入文件夹名称',
        showInput: true,
        defaultInputText: file.split('/').pop(),
        buttons: [
          {
            content: '添加',
            type: 'tint',
            operation() {
              if (modal.inputValue.value?.trim()) {
                modal.close(true)
              } else {
                sideTip.show('请输入合理的文件夹名称！', 'warn')
              }
            },
          },
          {
            content: '取消',
            type: 'warn',
          },
        ],
      })
      if (result) {
        let repoUpdate = await invoke<Repository[]>('add_new_repository', {
          new_repo_path: file,
          new_repo_name: result.input,
        })
        this.repositires = repoUpdate
      }
    },
  },
})
