import type GameInstance from '@/types/gameInstance'
import type { pluginType } from '../util/gameInfo'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'

export const useSelectedInstance = defineStore('selected-instance', () => {
  const instance_info = ref<GameInstance>()

  const plugins = computed<pluginType[]>(
    () => Object.keys(instance_info.value?.pluginsVersion || {}) as pluginType[],
  )

  return {
    instance_info,
    plugins,
  }
})
