import { isTauri, convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { getCurrentInstance, onMounted, onUnmounted, ref, watch } from 'vue'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { pclText } from '@/api/pclLocale'
import sideTip from '@/composables/sideTip'
import {
  deleteInstanceResources,
  importInstanceResources,
  listInstanceResources,
  openInstanceResourceFolder,
  setInstanceResourcesEnabled,
  type InstanceResourceEntry,
  type InstanceResourceKind,
} from '@/api/instanceResources'

interface ResourceMessages {
  deleted: string
  deletedMultiple: string
  deleteFailed: string
  toggleFailed: string
}

const defaultMessages: ResourceMessages = {
  deleted: 'Instance.Resource.Delete.RecycleSingle',
  deletedMultiple: 'Instance.Resource.Delete.RecycleMultiple',
  deleteFailed: 'Instance.Resource.Delete.Failed',
  toggleFailed: 'Instance.Resource.Ed.ToggleFailed',
}

const dialogFilters: Record<InstanceResourceKind, { name: string; extensions: string[] }[]> = {
  mod: [{ name: 'Mod', extensions: ['jar', 'zip', 'litemod'] }],
  resourcepack: [{ name: 'ResourcePack', extensions: ['zip'] }],
  shader: [{ name: 'Shader', extensions: ['zip'] }],
  schematic: [{ name: 'Schematic', extensions: ['schematic', 'schem', 'litematic', 'nbt'] }],
  screenshot: [{ name: 'Image', extensions: ['png'] }],
  save: [{ name: 'Archive', extensions: ['zip'] }],
  datapack: [{ name: 'Datapack', extensions: ['zip'] }],
}

export function useInstanceResources(options: {
  kind: () => InstanceResourceKind
  save?: () => string | undefined
  messages?: Partial<ResourceMessages>
}) {
  const instance = useSelectedInstance()
  const messages = { ...defaultMessages, ...options.messages }
  const items = ref<InstanceResourceEntry[]>([])
  const loading = ref(false)
  const error = ref('')
  let generation = 0

  const directory = () => instance.instance_info?.directory
  const save = () => options.save?.()

  // Datapacks need the owning save folder; nothing can be listed or changed without one.
  const missingSave = () => Boolean(options.save) && !save()

  async function refresh(silent = false) {
    const path = directory()
    const revision = ++generation
    if (!silent) items.value = []
    error.value = ''
    if (!isTauri() || !path || missingSave()) return
    if (!silent) loading.value = true
    try {
      const result = await listInstanceResources(path, options.kind(), save())
      if (revision === generation) items.value = result
    } catch (failure) {
      if (revision === generation) error.value = String(failure)
    } finally {
      if (revision === generation) loading.value = false
    }
  }

  async function openFolder() {
    const path = directory()
    if (!path || !isTauri() || missingSave()) return
    try {
      await openInstanceResourceFolder(path, options.kind(), save())
    } catch (failure) {
      sideTip.show(`${pclText('Instance.Resource.Error.OperationFailed')} ${failure}`, 'warn')
    }
  }

  async function importFiles() {
    const path = directory()
    if (!path || !isTauri() || missingSave()) return
    try {
      const picked = await open({
        multiple: true,
        directory: false,
        filters: dialogFilters[options.kind()],
      })
      if (!picked) return
      const sources = Array.isArray(picked) ? picked : [picked]
      if (!sources.length) return
      await importInstanceResources(path, options.kind(), sources, save())
      await refresh(true)
    } catch (failure) {
      sideTip.show(`${pclText('Instance.Resource.Error.OperationFailed')} ${failure}`, 'warn')
    }
  }

  async function setEnabled(entries: InstanceResourceEntry[], enabled: boolean) {
    const path = directory()
    if (!path || !isTauri() || !entries.length) return
    try {
      await setInstanceResourcesEnabled(
        path,
        entries.map((entry) => entry.path),
        enabled,
      )
      await refresh(true)
    } catch (failure) {
      sideTip.show(`${pclText(messages.toggleFailed)} ${failure}`, 'warn')
    }
  }

  async function remove(entries: InstanceResourceEntry[]) {
    const path = directory()
    if (!path || !isTauri() || !entries.length) return
    try {
      await deleteInstanceResources(
        path,
        entries.map((entry) => entry.path),
      )
      sideTip.show(
        entries.length === 1
          ? pclText(messages.deleted, [entries[0]?.name ?? ''])
          : pclText(messages.deletedMultiple, [entries.length]),
        'success',
      )
      await refresh(true)
    } catch (failure) {
      sideTip.show(`${pclText(messages.deleteFailed)} ${failure}`, 'warn')
    }
  }

  const toggle = (entry: InstanceResourceEntry) => setEnabled([entry], !entry.enabled)

  watch(
    () => [directory(), options.kind(), save()],
    () => void refresh(),
    { immediate: true },
  )
  onMounted(() => {
    if (!instance.instance_info) void instance.fetch().catch(() => {})
  })
  if (getCurrentInstance()) onUnmounted(() => generation++)

  return { items, loading, error, refresh, openFolder, importFiles, setEnabled, remove, toggle }
}

/** Picture token from the backend, turned into a URL the scoped protocol can serve. */
export function resourceUrl(token?: string) {
  return token && isTauri() ? convertFileSrc(token, 'pclinstance') : undefined
}
