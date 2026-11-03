import { ref, watch, computed, getCurrentInstance, onMounted, onUnmounted, type Ref } from 'vue'
import { isTauri } from '@tauri-apps/api/core'
import {
  memoryMibToTick,
  memoryTickToMib,
  previewMemorySettings,
  memorySettingsStorage,
  type MemorySettings,
  type MemoryPreview,
} from '@/api/memorySettings'

export function useMemorySettings(
  directory?: Ref<string | undefined>,
  is64Bit?: Ref<boolean | null>,
) {
  const mode = ref<'inherit' | 'auto' | 'custom'>(directory ? 'inherit' : 'custom')
  const maximum = ref(15)
  const initial = ref(0)
  const ready = ref(false)
  const error = ref<string | null>(null)
  const preview = ref<MemoryPreview | null>(null)
  const segments = computed(() => {
    if (!preview.value) return { used: 0, game: 0, empty: 100 }
    const {
      total_bytes: total,
      available_bytes: available,
      allocation_mib: allocation,
    } = preview.value
    const used = ((total - available) / total) * 100
    const game = (Math.min(available, (allocation ?? 0) * 1024 * 1024) / total) * 100
    return { used, game, empty: Math.max(0, 100 - used - game) }
  })
  let generation = 0
  let saveRevision = 0
  let previewRevision = 0
  let timer: ReturnType<typeof setTimeout> | undefined
  let interval: ReturnType<typeof setInterval> | undefined
  async function load() {
    const current = ++generation
    previewRevision++
    preview.value = null
    const path = directory ? directory.value : null
    ready.value = false
    error.value = null
    if (!isTauri() || path === undefined) return
    try {
      const { settings: result, error: saveError } = await memorySettingsStorage.read(path)
      if (current !== generation) return
      mode.value = result.settings?.mode || 'inherit'
      maximum.value = memoryMibToTick(result.custom.maximum_mib)
      initial.value =
        result.custom.initial_mib !== null ? memoryMibToTick(result.custom.initial_mib) : 0
      error.value = saveError
      ready.value = true
    } catch (failure) {
      if (current === generation) error.value = String(failure)
    }
  }
  function snapshot() {
    const max = memoryTickToMib(maximum.value)
    // 初始堆不得超过最大堆：这条规则只在这里算一次，settings 与 custom 共用同一个结果。
    const custom = {
      maximum_mib: max,
      initial_mib: initial.value === 0 ? null : Math.min(max, memoryTickToMib(initial.value)),
    }
    const settings: MemorySettings | null =
      mode.value === 'inherit'
        ? null
        : mode.value === 'auto'
          ? { mode: 'auto' }
          : { mode: 'custom', maximum_mib: max, initial_mib: custom.initial_mib }
    return { settings, custom }
  }
  function save() {
    const path = directory ? directory.value : null
    if (!ready.value || path === undefined) return
    const current = generation
    const revision = ++saveRevision
    error.value = null
    void memorySettingsStorage.write(snapshot(), path).catch((failure) => {
      if (current === generation && revision === saveRevision) error.value = String(failure)
    })
  }
  function refreshPreview() {
    const revision = ++previewRevision
    const path = directory ? directory.value : null
    if (!isTauri() || !ready.value || path === undefined) {
      preview.value = null
      return
    }
    void previewMemorySettings(path, snapshot().settings, is64Bit?.value ?? null)
      .then((result) => {
        if (revision === previewRevision) preview.value = result
      })
      .catch((failure) => {
        if (revision === previewRevision) preview.value = null
        console.warn('Unable to read memory preview', failure)
      })
  }
  if (getCurrentInstance()) {
    onMounted(() => {
      watch(
        [
          mode,
          maximum,
          initial,
          ready,
          ...(directory ? [directory] : []),
          ...(is64Bit ? [is64Bit] : []),
        ],
        () => {
          clearTimeout(timer)
          timer = setTimeout(refreshPreview, 150)
        },
        { immediate: true },
      )
      interval = setInterval(() => {
        if (document.hidden) return
        refreshPreview()
      }, 5000)
    })
    onUnmounted(() => {
      previewRevision++
      clearTimeout(timer)
      clearInterval(interval)
    })
  }
  if (directory) watch(directory, load, { immediate: true, flush: 'sync' })
  else void load()
  return { mode, maximum, initial, ready, error, preview, segments, load, save }
}

export function memoryGiB(bytes: number) {
  return `${(bytes / 1073741824).toFixed(1)} GiB`
}
