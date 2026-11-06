import { invoke } from '@tauri-apps/api/core'
import { pclText } from '@/api/pclLocale'
import type { JavaLaunchInfo } from '@/api/javaReq'
import { useModal } from '@/composables/useModal'
import type { LoaderInstall } from '@/stores/task'

export async function prepareLoaderJava(game: string, loader: LoaderInstall | null | undefined) {
  if (!loader || !['forge', 'neoforge'].includes(loader.family)) return true
  const offer = await invoke<Pick<JavaLaunchInfo, 'requirements' | 'download_major'> | null>(
    'prepare_forgelike_java',
    { version_id: game, loader },
  )
  if (!offer) return true
  const modal = useModal()
  if (!offer.download_major) {
    await modal.open({
      title: pclText('Minecraft.Launch.Java.NotFound.Title'),
      content: pclText('Proto.Java.ManualRequired', [
        offer.requirements.min[0],
        offer.requirements.max[0],
      ]),
      buttons: [
        {
          content: pclText('Common.Action.Close'),
          operation: () => modal.close(),
        },
      ],
    })
    return false
  }
  const answer = await modal.open({
    title: pclText('Minecraft.Launch.Java.AutoDownload.Title'),
    content: pclText('Minecraft.Launch.Java.AutoDownload.Message', [
      `Java ${offer.download_major}`,
    ]),
    buttons: [
      {
        content: pclText('Minecraft.Launch.Java.AutoDownload.Action'),
        operation: () => modal.close(true),
      },
      {
        content: pclText('Common.Action.Cancel'),
        operation: () => modal.close(false),
      },
    ],
  })
  if (answer.confirmed) loader.install_java = true
  return answer.confirmed
}
