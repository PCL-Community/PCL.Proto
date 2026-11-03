import { useRouter } from 'vue-router'
import { invoke, isTauri } from '@tauri-apps/api/core'
import { useModal } from '@/composables/useModal'
import sideTip from '@/composables/sideTip'
import { openExternal } from '@/util/externalLink'
import { homepageRevision } from '@/api/customHomepage'
import { pclText } from '@/api/pclLocale'
import { useSelectedInstance } from '@/stores/gameLaunch'
import { useLaunchState } from '@/stores/launchState'
import { homepageRoute } from '@/util/homepageXaml'
import { useRepositoriesStore } from '@/stores/repositories'
import type GameInstance from '@/types/gameInstance'

export function useHomepageActions(preview = false) {
  const router = useRouter()
  const modal = useModal()
  const selected = useSelectedInstance()
  const launchState = useLaunchState()
  return (type: string, data: string) => {
    if (preview) {
      sideTip.show(pclText('Proto.Homepage.PreviewEvent', [type, data]))
      return
    }
    void (async () => {
      const args = data.split('|')
      const first = args[0] || ''
      switch (type) {
        case 'OpenUrl':
          if (!(await openExternal(data))) throw new Error(pclText('Proto.Error.OpenUrl'))
          break
        case 'CopyText':
          await navigator.clipboard.writeText(data)
          sideTip.show(pclText('Common.Hint.Copied'), 'success')
          break
        case 'ShowDialog':
          await modal.open({
            title: first.replaceAll('\\n', '\n'),
            content: (args[1] || '').replaceAll('\\n', '\n'),
            buttons: [
              {
                content: args[2] || pclText('Common.Action.Confirm'),
                operation: () => modal.close(),
              },
            ],
          })
          break
        case 'ShowHint':
          sideTip.show(
            first.replaceAll('\\n', '\n'),
            /^(success|finish)$/i.test(args[1] || '')
              ? 'success'
              : /^(error|critical)$/i.test(args[1] || '')
                ? 'warn'
                : 'default',
          )
          break
        case 'RefreshHomepage':
        case 'RefreshPage':
          homepageRevision.value++
          break
        case 'DailyFortune':
          if (!isTauri()) throw new Error(pclText('Proto.Homepage.NativeOnly'))
          sideTip.show(pclText('Proto.Tools.Luck', [await invoke<number>('get_lucky_today')]))
          break
        case 'LaunchGame': {
          let instance = selected.instance_info
          if (!isTauri()) throw new Error(pclText('Proto.Homepage.NativeOnly'))
          if (first && first !== instance?.name && first !== instance?.id) {
            const repositories = useRepositoriesStore()
            await repositories.fetchFromBackend()
            const index =
              repositories.repositories?.findIndex((repo) => repo.path === instance?.global_dir) ??
              -1
            const indices = index >= 0 ? [index] : repositories.repositories?.map((_, i) => i) || []
            const matches: { instance: GameInstance; index: number }[] = []
            for (const index of indices) {
              const instances = await repositories.getInstancesInRepository(index)
              for (const candidate of instances) {
                if (candidate.name === first || candidate.id === first)
                  matches.push({ instance: candidate, index })
              }
            }
            if (matches.length === 1) {
              instance = await invoke<GameInstance>('select_instance', {
                repository_index: matches[0]!.index,
                instance_name: matches[0]!.instance.name,
              })
              selected.setInstance(instance)
            } else {
              await router.push('/instance_select')
              throw new Error(pclText('Proto.Homepage.SelectInstance', [first]))
            }
          }
          if (!instance) {
            await router.push('/instance_select')
            return
          }
          await router.push('/home')
          launchState.pendingServer = { directory: instance.directory, address: args[1] || '' }
          break
        }
        case 'SwitchPage': {
          const target = homepageRoute(data)
          if (!router.resolve(target).matched.length) throw new Error(`Unsupported page: ${target}`)
          await router.push(target)
          break
        }
      }
    })().catch((error) => sideTip.show(String(error), 'warn'))
  }
}
