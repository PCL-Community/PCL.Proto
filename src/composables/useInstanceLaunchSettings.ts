import { isTauri } from '@tauri-apps/api/core'
import { getCurrentInstance, onUnmounted, reactive, ref, watch, type Ref } from 'vue'
import {
  emptyInstanceLaunchSettings,
  instanceLaunchSettingsStorage,
  type InstanceLaunchSettings,
} from '@/api/instanceLaunchSettings'

export function useInstanceLaunchSettings(directory?: Ref<string | undefined>) {
  const settings = reactive<InstanceLaunchSettings>(emptyInstanceLaunchSettings())
  const ready = ref(false)
  const error = ref<string | null>(null)
  let generation = 0
  let saveRevision = 0
  const currentPath = () => (directory ? directory.value : undefined)

  async function load() {
    const revision = ++generation
    const path = currentPath()
    ready.value = false
    error.value = null
    Object.assign(settings, emptyInstanceLaunchSettings())
    if (!isTauri() || !path) return
    try {
      const saved = await instanceLaunchSettingsStorage.read(path)
      if (revision !== generation) return
      Object.assign(settings, saved.settings)
      error.value = saved.error
      ready.value = true
    } catch (failure) {
      if (revision === generation) error.value = String(failure)
    }
  }

  function save() {
    const path = currentPath()
    if (!ready.value || !path) return
    const revision = generation
    const edit = ++saveRevision
    error.value = null
    void instanceLaunchSettingsStorage.write({ ...settings }, path).catch((failure) => {
      if (revision === generation && edit === saveRevision) error.value = String(failure)
    })
  }

  if (directory) watch(directory, () => void load(), { immediate: true })
  else void load()
  if (getCurrentInstance())
    onUnmounted(() => {
      generation++
    })
  return { settings, ready, error, load, save }
}
