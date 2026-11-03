import { isTauri } from '@tauri-apps/api/core'
import { onMounted, ref } from 'vue'
import { getProxySettings, setProxySettings, type ProxyMode } from '@/api/networkSettings'

export function useProxySettings() {
  const mode = ref<ProxyMode>('off')
  const address = ref('')
  const ready = ref(false)
  const busy = ref(false)
  const error = ref<string | null>(null)

  async function load() {
    if (!isTauri()) return
    try {
      const settings = await getProxySettings()
      mode.value = settings.mode
      address.value = settings.address
      error.value = null
    } catch (failure) {
      error.value = String(failure)
    } finally {
      ready.value = true
    }
  }

  async function save() {
    if (!isTauri() || busy.value) return
    busy.value = true
    try {
      const settings = await setProxySettings({ mode: mode.value, address: address.value.trim() })
      mode.value = settings.mode
      address.value = settings.address
      error.value = null
    } catch (failure) {
      error.value = String(failure)
    } finally {
      busy.value = false
    }
  }

  onMounted(load)
  return { mode, address, ready, busy, error, load, save }
}
