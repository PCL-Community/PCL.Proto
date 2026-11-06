import { invoke } from '@tauri-apps/api/core'
import { pclText } from '@/api/pclLocale'
import { useModal } from '@/composables/useModal'
import sideTip from '@/composables/sideTip'
import { isTaskCancelled, useTaskManager, type ModpackInfo } from '@/stores/task'
import type { IVersion } from '@/types/comp'
import { prepareLoaderJava } from './loaderJava'

async function chooseName(defaultName: string) {
  let value = defaultName
    .replace(/[\\/:*?"<>|\x00-\x1f]/g, ' ')
    .trim()
    .replace(/\.+$/, '')
  while (true) {
    const choice = await useModal().open({
      title: pclText('Minecraft.Download.Modpack.InputInstanceName'),
      showInput: true,
      defaultInputText: value,
    })
    if (!choice.confirmed) return undefined
    value = choice.input?.trim() || ''
    if (
      value &&
      !/[\\/:*?"<>|\x00-\x1f]/.test(value) &&
      !value.endsWith('.') &&
      !/^(con|prn|aux|nul|com[1-9]|lpt[1-9])(?:\.|$)/i.test(value)
    )
      return value
    sideTip.show(pclText('Proto.Instance.InvalidName'), 'warn')
  }
}

async function chooseOptional(info: ModpackInfo) {
  const selected: string[] = []
  for (const file of info.optional_files) {
    const choice = await useModal().open({
      title: pclText('Minecraft.Download.Modpack.OptionalFile.Title'),
      content: pclText('Minecraft.Download.Modpack.OptionalFile.Message', [file]),
      confirmText: pclText('Minecraft.Download.Modpack.OptionalFile.Download'),
      cancelText: pclText('Minecraft.Download.Modpack.OptionalFile.Skip'),
    })
    if (choice.confirmed) selected.push(file)
  }
  return selected
}

export async function importLocalModpack(path: string) {
  const info = await invoke<ModpackInfo>('inspect_modpack', { path })
  const name = await chooseName(info.name)
  if (!name) return
  const optional = await chooseOptional(info)
  if (!(await prepareLoaderJava(info.minecraft, info.loader))) return
  sideTip.show(pclText('Download.Comp.QuickDownload.Hint.DownloadStarted', [name]))
  try {
    await useTaskManager().installModpack(path, name, optional, info.loader)
    sideTip.show(pclText('Instance.Resource.Install.SuccessSingle', [name]), 'success')
  } catch (error) {
    if (!isTaskCancelled(error)) throw error
  }
}

export async function installOnlineModpack(file: IVersion['files'][number], defaultName: string) {
  const name = await chooseName(defaultName)
  if (!name) return
  const tasks = useTaskManager()
  sideTip.show(pclText('Download.Comp.QuickDownload.Hint.DownloadStarted', [name]))
  let token: string | undefined
  try {
    const prepared = await tasks.prepareOnlineModpack(name, {
      url: file.url,
      size: file.size,
      sha1: file.hashes?.sha1 || '',
    })
    token = prepared.token
    const optional = await chooseOptional(prepared.info)
    if (!(await prepareLoaderJava(prepared.info.minecraft, prepared.info.loader))) return
    await tasks.installOnlineModpack(token, name, optional, prepared.info.loader)
    sideTip.show(pclText('Instance.Resource.Install.SuccessSingle', [name]), 'success')
  } catch (error) {
    if (!isTaskCancelled(error)) throw error
  } finally {
    if (token) await invoke('release_online_modpack', { token }).catch(console.warn)
  }
}
