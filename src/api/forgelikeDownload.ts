import { invoke } from '@tauri-apps/api/core'
import { save } from '@tauri-apps/plugin-dialog'
import { pclText } from '@/api/pclLocale'
import { useTaskManager, type LoaderInstall } from '@/stores/task'

export async function downloadForgelike(loaderName: string, game: string, version: string) {
  const loader: LoaderInstall = {
    family: loaderName === 'Forge' ? 'forge' : 'neoforge',
    version,
  }
  const filename = await invoke<string>('forgelike_installer_name', {
    game,
    loader,
  })
  const extension = filename.endsWith('.zip') ? 'zip' : 'jar'
  const destination = await save({
    title: pclText('Download.Version.SelectSaveLocation'),
    defaultPath: filename,
    filters: [{ name: extension.toUpperCase(), extensions: [extension] }],
  })
  if (!destination) return null
  await useTaskManager().downloadForgelikeInstaller(
    `${loaderName} ${game} - ${version}`,
    game,
    loader,
    destination,
  )
  return destination
}
