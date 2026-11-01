import { pclText } from '@/api/pclLocale'
import { FloatButtonType, useFloatButton } from '@/composables/useFloatButton'
import { Channel, invoke } from '@tauri-apps/api/core'
import { defineStore } from 'pinia'
import { computed, ref } from 'vue'
import { useSelectedInstance } from '@/stores/gameLaunch'
import sideTip from '@/composables/sideTip'

export enum TaskStatus {
  Pending = 0,
  Running = 1,
  Completed = 2,
  Failed = 3,
}
type DownloadKind = 'install' | 'client' | 'server' | 'resource' | 'custom'

export interface ITaskItem {
  id: number
  task_id: number
  name: string
  status: TaskStatus
  speed: number | null
  progress: number
  remaining: number | undefined
}

interface Task {
  id: number
  versionId: string
  name: string
  kind: DownloadKind
  status: TaskStatus
  speed: number
  items: ITaskItem[]
  progress: number
  error: string
}

interface TaskItemReport {
  item_id: number
  task_id: number
  files_remaining: number
  progress: number
  status: TaskStatus
  speed: number | null
}

function createTask(id: number, versionId: string, kind: DownloadKind): Task {
  const keys =
    kind === 'resource' || kind === 'custom'
      ? ['Download.Comp.Detail.DownloadFile']
      : [
          'Minecraft.Download.Stage.VanillaJsonDownload',
          kind === 'server'
            ? 'Minecraft.Download.Stage.DownloadServerFile'
            : 'Minecraft.Download.Stage.DownloadCoreJar',
        ]
  if (kind === 'install')
    keys.push(
      'Minecraft.Download.Stage.VanillaLibrariesDownload',
      'Minecraft.Download.Stage.DownloadVanillaAssets',
    )
  return {
    id,
    versionId,
    kind,
    get name() {
      if (kind === 'custom') return pclText('Tools.Test.CustomDownload.LoaderName', [versionId])
      if (kind === 'resource')
        return pclText('Download.Comp.Detail.DownloadResource', ['', versionId])
      return pclText(
        kind === 'server'
          ? 'Minecraft.Download.Stage.MinecraftServerDownload'
          : 'Minecraft.Download.Stage.MinecraftDownload',
        [versionId],
      )
    },
    status: TaskStatus.Running,
    speed: 0,
    progress: 0,
    error: '',
    items: keys.map((key, itemId) => ({
      id: itemId,
      task_id: id,
      get name() {
        return pclText(key)
      },
      status: TaskStatus.Pending,
      speed: 0,
      progress: 0,
      remaining: undefined,
    })),
  }
}

function isActive(task: Task) {
  return task.status === TaskStatus.Pending || task.status === TaskStatus.Running
}

export const useTaskManager = defineStore('task-manager', () => {
  const tasks = ref<Task[]>([])
  let nextTaskId = 0
  const activeTasks = computed(() => tasks.value.filter(isActive))
  const totalProgress = computed(() => {
    const visible = activeTasks.value.length ? activeTasks.value : tasks.value
    return (
      (100 * visible.reduce((sum, task) => sum + task.progress, 0)) /
      (visible.length || 1)
    ).toFixed(2)
  })
  const totalSpeed = computed(() => activeTasks.value.reduce((sum, task) => sum + task.speed, 0))
  const totalRemaining = computed(() =>
    activeTasks.value.reduce(
      (sum, task) => sum + task.items.reduce((count, item) => count + (item.remaining || 0), 0),
      0,
    ),
  )
  const activeTaskCount = computed(() => activeTasks.value.length)
  const floatButton = useFloatButton()

  async function runDownload(
    versionId: string,
    command: string,
    args: Record<string, unknown>,
    kind: DownloadKind,
  ) {
    const id = nextTaskId++
    tasks.value.push(createTask(id, versionId, kind))
    const task = tasks.value.find((item) => item.id === id)!
    floatButton.setFloatButton(FloatButtonType.TaskManage)
    floatButton.floatButtonState.visible = true
    const onEvent = new Channel<TaskItemReport>()
    onEvent.onmessage = (message) => {
      if (message.task_id !== id || !isActive(task)) return
      const item = task.items.find((item) => item.id === message.item_id)
      if (!item) return
      item.status = message.status
      item.progress = Number.isFinite(message.progress)
        ? Math.max(0, Math.min(1, message.progress))
        : 0
      item.remaining = message.files_remaining
      item.speed =
        message.status === TaskStatus.Running && Number.isFinite(message.speed)
          ? Math.max(0, message.speed || 0)
          : 0
      task.progress = task.items.reduce((sum, item) => sum + item.progress, 0) / task.items.length
      task.speed = task.items.reduce((sum, item) => sum + (item.speed || 0), 0)
      // File stages can finish before native extraction and instance publication.
      // Only the command result commits the overall task to a terminal state.
    }
    try {
      const result = await invoke<
        string | { selected: boolean; selection_error: string | null } | void
      >(command, {
        ...args,
        on_event: onEvent,
        version_id: versionId,
        task_id: id,
      })
      task.status = TaskStatus.Completed
      task.progress = 1
      task.items.forEach((item) => {
        item.status = TaskStatus.Completed
        item.progress = 1
        item.remaining = 0
        item.speed = 0
      })
      if (kind === 'install' && result && typeof result === 'object') {
        if (result.selection_error)
          sideTip.show(pclText('Proto.Install.SelectionFailed', [result.selection_error]), 'warn')
        if (result.selected)
          void useSelectedInstance()
            .fetch()
            .catch((error) =>
              sideTip.show(pclText('Proto.Install.SelectionFailed', [String(error)]), 'warn'),
            )
      }
      return result
    } catch (error) {
      task.status = TaskStatus.Failed
      task.error = String(error)
      task.items.forEach((item) => {
        if (item.status !== TaskStatus.Completed) item.status = TaskStatus.Failed
        item.speed = 0
      })
      throw error
    } finally {
      task.speed = 0
      if (
        !activeTaskCount.value &&
        floatButton.floatButtonState.type === FloatButtonType.TaskManage
      )
        floatButton.floatButtonState.visible = false
    }
  }

  function startDownloadMCVersion(
    versionId: string,
    instanceName: string,
    fabric?: { family: 'fabric' | 'legacyfabric'; version: string },
  ) {
    return runDownload(
      versionId,
      'download_minecraft_version',
      { instance_name: instanceName, fabric: fabric || null },
      'install',
    )
  }

  function exportMinecraftVersion(versionId: string, directory: string, server: boolean) {
    const kind = server ? 'server' : 'client'
    if (activeTasks.value.some((task) => task.versionId === versionId && task.kind === kind))
      throw new Error(
        pclText(
          server
            ? 'Minecraft.Download.Error.ServerDownloading'
            : 'Minecraft.Download.Error.InstanceDownloading',
        ),
      )
    return runDownload(versionId, 'export_minecraft_version', { directory, server }, kind)
  }

  function downloadResource(
    name: string,
    info: { url: string; size: number; sha1: string },
    destination: string,
  ) {
    return runDownload(
      name,
      'download_resource',
      { info: { ...info, path: null }, destination },
      'resource',
    )
  }

  function downloadCustom(name: string, url: string, destination: string) {
    return runDownload(name, 'download_custom_file', { url, destination }, 'custom')
  }

  return {
    tasks,
    totalProgress,
    totalSpeed,
    totalRemaining,
    activeTaskCount,
    startDownloadMCVersion,
    exportMinecraftVersion,
    downloadResource,
    downloadCustom,
  }
})
