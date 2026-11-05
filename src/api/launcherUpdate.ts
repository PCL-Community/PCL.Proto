import { ref } from 'vue'
import { pclText } from './pclLocale'

export const launcherUpdate = ref<{ version: string; body: string; url: string }>()
export const launcherUpdateChecking = ref(false)
export const launcherUpdateError = ref<string | null>(null)

let checkPromise: Promise<void> | undefined

export function checkLauncherUpdate() {
  if (checkPromise) return checkPromise

  launcherUpdateChecking.value = true
  launcherUpdateError.value = null
  checkPromise = (async () => {
    const repository = 'https://github.com/PCL-Community/PCL.Proto'
    try {
      const response = await fetch(
        'https://api.github.com/repos/PCL-Community/PCL.Proto/releases/latest',
      )
      if (!response.ok) throw new Error(`HTTP ${response.status}`)
      const data: unknown = await response.json()
      if (
        !data ||
        typeof data !== 'object' ||
        !('tag_name' in data) ||
        typeof data.tag_name !== 'string' ||
        !('html_url' in data) ||
        typeof data.html_url !== 'string' ||
        !data.html_url.startsWith(`${repository}/releases/`)
      )
        throw new Error(pclText('Proto.Update.InvalidResponse'))

      launcherUpdate.value = {
        version: data.tag_name,
        url: data.html_url,
        body: 'body' in data && typeof data.body === 'string' ? data.body : '',
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
