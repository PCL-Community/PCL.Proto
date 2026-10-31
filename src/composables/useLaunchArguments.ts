import { isTauri } from '@tauri-apps/api/core'
import { getCurrentInstance, onUnmounted, ref, watch, type Ref } from 'vue'
import {
  readLaunchArguments,
  saveLaunchArguments,
  stageLaunchArguments,
} from '@/api/launchArguments'

export function useLaunchArguments(directory?: Ref<string | undefined>) {
  const jvm = ref('')
  const game = ref('')
  const classpath = ref('')
  const ready = ref(false)
  const error = ref('')
  let generation = 0
  let hydrating = false
  let dirty = false
  let editRevision = 0
  const currentPath = () => (directory ? directory.value : null)
  async function load() {
    const revision = ++generation
    const path = currentPath()
    ready.value = false
    error.value = ''
    dirty = false
    if (!isTauri() || path === undefined) return
    try {
      const result = await readLaunchArguments(path)
      if (revision !== generation) return
      hydrating = true
      jvm.value = result.settings.jvm
      game.value = result.settings.game
      classpath.value = result.settings.classpath
      hydrating = false
      error.value = result.error
      dirty = result.dirty
      ready.value = true
    } catch (failure) {
      if (revision === generation) error.value = String(failure)
    }
  }
  function save(path = currentPath()) {
    if (path === undefined || !dirty) return
    const revision = generation
    const edit = editRevision
    dirty = false
    void saveLaunchArguments(path).catch((failure) => {
      if (revision === generation && edit === editRevision && path === currentPath()) {
        error.value = String(failure)
        dirty = true
      }
    })
  }
  watch(
    [jvm, game, classpath],
    () => {
      const path = currentPath()
      if (hydrating || !ready.value || path === undefined) return
      stageLaunchArguments(path, { jvm: jvm.value, game: game.value, classpath: classpath.value })
      editRevision++
      dirty = true
      error.value = ''
    },
    { flush: 'sync' },
  )
  if (directory)
    watch(
      directory,
      (path, old) => {
        if (old && dirty) save(old)
        void load()
      },
      { immediate: true, flush: 'sync' },
    )
  else void load()
  if (getCurrentInstance())
    onUnmounted(() => {
      save()
      generation++
    })
  return { jvm, game, classpath, ready, error, load, save }
}
