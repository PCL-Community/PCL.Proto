import { invoke, isTauri } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'
import { pclLocale, pclText } from './pclLocale'
import { getMinecraftChangelogUrl } from './gameVersions'
import { isTaskCancelled, useTaskManager } from '@/stores/task'
import sideTip from '@/composables/sideTip'
import { openExternal } from '@/util/externalLink'

export async function saveMinecraftVersion(versionId: string, server = false) {
  if (!isTauri()) {
    sideTip.show(pclText('Proto.Desktop.Download'), 'warn')
    return
  }
  try {
    const directory = await open({ directory: true, multiple: false })
    if (!directory) return
    const download = useTaskManager().exportMinecraftVersion(versionId, directory, server)
    sideTip.show(pclText('Download.Comp.QuickDownload.Hint.DownloadStarted', [versionId]))
    const path = await download
    sideTip.show(pclText('Proto.Download.Saved', [path]))
  } catch (error) {
    if (isTaskCancelled(error)) return
    sideTip.show(pclText('Proto.Error.Download', [pclText(String(error), [versionId])]), 'warn')
  }
}

export async function openMinecraftChangelog(versionId: string) {
  const url = getMinecraftChangelogUrl(versionId, pclLocale.value)
  try {
    if (isTauri()) await invoke('open_minecraft_wiki', { url })
    else void openExternal(url)
  } catch (error) {
    sideTip.show(
      `${pclText('Instance.Resource.Item.OpenChangelogFailed')}: ${String(error)}`,
      'warn',
    )
  }
}
