import type GameInstance from '@/types/gameInstance'
import type { pluginType } from '../types/gameInfo'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { invoke } from '@tauri-apps/api/core'

export const useSelectedInstance = defineStore('selected-instance', () => {
  const instance_info = ref<GameInstance | null>(null)

  const plugins = computed<pluginType[]>(
    () => Object.keys(instance_info.value?.pluginsVersion || {}) as pluginType[],
  )

  async function fetch() {
    instance_info.value = await invoke<GameInstance | null>('get_active_instance')
  }

  return {
    instance_info,
    plugins,
    fetch,
  }
})
