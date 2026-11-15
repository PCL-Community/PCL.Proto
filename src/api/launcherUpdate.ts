import { computed, ref } from 'vue'
import { pclText } from './pclLocale'
import { isNewerRelease } from '@/util/releaseVersion'

export const launcherUpdate = ref<{ version: string; body: string; url: string }>()
export const launcherUpdateChecking = ref(false)
export const launcherUpdateError = ref<string | null>(null)
export const launcherUpdateAvailable = computed(() =>
  launcherUpdate.value ? isNewerRelease(launcherUpdate.value.version, __APP_VERSION__) : false,
)

let checkPromise: Promise<void> | undefined

export function checkLauncherUpdate() {
  if (checkPromise) return checkPromise

  launcherUpdateChecking.value = true
  launcherUpdateError.value = null
  checkPromise = (async () => {
    const repository = 'https://github.com/PCL-Community/PCL.Proto'
    const api = 'https://api.github.com/repos/PCL-Community/PCL.Proto'
    try {
      const response = await fetch(`${api}/releases?per_page=30`)
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const data: unknown = await response.json()
      if (!Array.isArray(data)) throw new Error(pclText('Proto.Update.InvalidResponse'))
      const release = data.find(
        (item): item is Record<string, unknown> =>
          Boolean(item) &&
          typeof item === 'object' &&
          !('draft' in item && item.draft === true) &&
          !('prerelease' in item && item.prerelease === true),
      )
      if (
        !release ||
        typeof release.tag_name !== 'string' ||
        typeof release.html_url !== 'string' ||
        !release.html_url.startsWith(`${repository}/releases/`)
      )
        throw new Error(pclText('Proto.Update.InvalidResponse'))

      launcherUpdate.value = {
        version: release.tag_name,
        url: release.html_url,
        body: typeof release.body === 'string' ? release.body : '',
      }
    } catch (error) {
      launcherUpdateError.value = String(error)
    } finally {
      launcherUpdateChecking.value = false
      checkPromise = undefined
    }
  })()

  return checkPromise
}
