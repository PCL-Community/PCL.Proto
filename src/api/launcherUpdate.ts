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

/** 0 = 自动下载并安装，1 = 自动下载并提示，2 = 仅提示，3 = 关闭自动检查。 */
export type UpdateMode = 0 | 1 | 2 | 3
export type UpdateChannel = 'release' | 'beta'

const CHANNEL_STORAGE_KEY = 'PCL.Update.Channel'
const MODE_STORAGE_KEY = 'PCL.Update.Mode'

function readStorage<T extends string | number>(key: string, fallback: T): T {
  const stored = localStorage.getItem(key)
  return stored === null ? fallback : (stored as T)
}

export const launcherUpdateChannel = ref<UpdateChannel>(
  readStorage<UpdateChannel>(CHANNEL_STORAGE_KEY, 'release'),
)
export const launcherUpdateMode = ref<UpdateMode>(
  Number(readStorage(MODE_STORAGE_KEY, '2')) as UpdateMode,
)

export function setLauncherUpdateChannel(channel: UpdateChannel) {
  launcherUpdateChannel.value = channel
  localStorage.setItem(CHANNEL_STORAGE_KEY, channel)
  launcherUpdate.value = undefined
  checkPromise = undefined
}

export function setLauncherUpdateMode(mode: UpdateMode) {
  launcherUpdateMode.value = mode
  localStorage.setItem(MODE_STORAGE_KEY, String(mode))
}

export function checkLauncherUpdate() {
  if (checkPromise) return checkPromise
  if (launcherUpdateMode.value === 3) return Promise.resolve()

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
          (launcherUpdateChannel.value === 'beta' ||
            !('prerelease' in item && item.prerelease === true)),
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
