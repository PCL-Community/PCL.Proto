import { invoke, isTauri } from '@tauri-apps/api/core'
import { join } from '@tauri-apps/api/path'
import { open, save } from '@tauri-apps/plugin-dialog'
import { pclText } from './pclLocale'
import { getCompVersions } from './compProjects'
import { compatibleResource, latestResourceVersion } from './compResources'
import type { CompType, ISearchHit, IVersion } from '@/types/comp'
import type GameInstance from '@/types/gameInstance'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { useRepositoriesStore } from '@/stores/repositories'
import { useTaskManager } from '@/stores/task'
import { useModal } from '@/composables/useModal'
import sideTip from '@/composables/sideTip'

type ResourceFile = IVersion['files'][number]
const folderKinds: Partial<Record<CompType, string>> = {
  mod: 'mod',
  resourcepack: 'resourcepack',
  shader: 'shader',
}

async function instanceDestination(instance: GameInstance, type: CompType, filename: string) {
  const kind = folderKinds[type]
  if (!kind) return undefined
  return invoke<string>('instance_resource_destination', {
    instanceDirectory: instance.directory,
    kind,
    filename,
  })
}

function start(file: ResourceFile, destination: string, name: string) {
  if (
    !file.filename ||
    file.filename === '.' ||
    file.filename === '..' ||
    /[\\/:*?"<>|\x00-\x1f]/.test(file.filename)
  )
    throw new Error('Invalid resource filename')
  sideTip.show(pclText('Download.Comp.QuickDownload.Hint.DownloadStarted', [name]), 'success')
  void useTaskManager()
    .downloadResource(
      name,
      { url: file.url, size: file.size, sha1: file.hashes?.sha1 || '' },
      destination,
    )
    .then(() => sideTip.show(pclText('Proto.Download.Saved', [destination]), 'success'))
    .catch((error) => sideTip.show(pclText('Proto.Error.Download', [String(error)]), 'warn'))
}

export async function saveResource(file: ResourceFile, version: IVersion, type: CompType) {
  if (!/^https:\/\//i.test(file.url)) throw new Error('Invalid resource URL')
  if (!isTauri()) {
    const link = document.createElement('a')
    link.href = file.url
    link.download = file.filename
    link.target = '_blank'
    link.rel = 'noopener noreferrer'
    link.click()
    return
  }
  const instance = useSelectedInstance().instance_info
  const defaultPath =
    instance && compatibleResource(version, instance, type)
      ? await instanceDestination(instance, type, file.filename)
      : undefined
  const destination = await save({ defaultPath: defaultPath || file.filename })
  if (destination) start(file, destination, version.name)
}

async function choose(options: { value: string; label: string }[], title: string) {
  const result = await useModal().open({ title, selectOptions: options })
  return result.confirmed ? result.selected : undefined
}

export async function quickDownload(project: ISearchHit) {
  if (!isTauri()) {
    sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
    return
  }
  sideTip.show(pclText('Download.Comp.QuickDownload.Hint.Loading'))
  const versions = await getCompVersions(project.project_id)
  if (!versions.length)
    return sideTip.show(pclText('Download.Comp.QuickDownload.Hint.NoFile'), 'warn')
  let behavior = '0'
  try {
    behavior = String(
      JSON.parse(localStorage.getItem('pcl-ui:PageSetupGameManage:global') || '{}')
        .ComboDownloadQuickBehavior ?? '0',
    )
  } catch {
    /* use default */
  }
  if (behavior === '0') {
    behavior =
      (await choose(
        ['CurrentInstance', 'AskInstance', 'AskPath'].map((key, index) => ({
          value: String(index + 1),
          label: pclText(`Download.Comp.QuickDownload.ChooseMethod.${key}`),
        })),
        pclText('Download.Comp.QuickDownload.ChooseMethod.Title'),
      )) || ''
  }
  if (!behavior) return
  let instance: GameInstance | undefined
  if (behavior === '1') {
    instance = useSelectedInstance().instance_info || undefined
    if (!instance)
      return sideTip.show(pclText('Download.Comp.QuickDownload.Hint.NoInstance'), 'warn')
  } else if (behavior === '2') {
    const repos = useRepositoriesStore()
    if (!repos.repositories) await repos.fetchFromBackend()
    const instances = (
      await Promise.all(
        (repos.repositories || []).map((_, index) => repos.getInstancesInRepository(index)),
      )
    )
      .flat()
      .filter((item) => latestResourceVersion(versions, item, project.project_type))
    if (!instances.length)
      return sideTip.show(pclText('Download.Comp.QuickDownload.Hint.NoCompatibleInstance'), 'warn')
    const current = useSelectedInstance().instance_info?.directory
    instances.sort(
      (a, b) =>
        Number(b.directory === current) - Number(a.directory === current) ||
        a.name.localeCompare(b.name),
    )
    const selected = await choose(
      instances.map((item) => ({ value: item.directory, label: `${item.name} (${item.version})` })),
      pclText('Download.Comp.QuickDownload.ChooseInstance.Title'),
    )
    instance = instances.find((item) => item.directory === selected)
    if (!instance) return
  }
  const version = latestResourceVersion(versions, instance, project.project_type)
  if (!version)
    return sideTip.show(
      pclText(
        instance
          ? 'Download.Comp.QuickDownload.Hint.NoCompatibleFile'
          : 'Download.Comp.QuickDownload.Hint.NoFile',
      ),
      'warn',
    )
  const file = version.files.find((item) => item.primary) || version.files[0]
  if (!file) return
  if (
    !file.filename ||
    file.filename === '.' ||
    file.filename === '..' ||
    /[\\/:*?"<>|\x00-\x1f]/.test(file.filename)
  )
    throw new Error('Invalid resource filename')
  let destination = instance
    ? await instanceDestination(instance, project.project_type, file.filename)
    : undefined
  if (!destination) {
    const directory = await open({
      directory: true,
      multiple: false,
      title: pclText('Download.Comp.QuickDownload.Hint.SelectFolder'),
    })
    if (!directory) return
    destination = await join(directory, file.filename)
  }
  start(file, destination, project.title)
}
