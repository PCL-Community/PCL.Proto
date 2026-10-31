import { invoke, isTauri } from '@tauri-apps/api/core'
import { ref } from 'vue'
import { maxLogLines } from '@/util/logLines'

export const logSlider = ref(13)
export const logSaveError = ref('')
let pending: Promise<void> = Promise.resolve()
let timer: ReturnType<typeof setTimeout> | undefined

function sliderForLines(lines: number) {
  if (lines === 0) return 29
  return Array.from({ length: 29 }, (_, i) => i).reduce((best, i) =>
    Math.abs(maxLogLines(i) - lines) < Math.abs(maxLogLines(best) - lines) ? i : best,
  )
}

function legacySlider() {
  for (const page of ['PageLogRight', 'PageSetupLauncherMisc']) {
    const raw = localStorage.getItem(`pcl-ui:${page}:global`)
    try {
      const value = JSON.parse(raw || '{}')?.SliderMaxLog
      if (typeof value === 'number' && Number.isInteger(value) && value >= 0 && value <= 29)
        return value
    } catch {
      // Keep the old data intact and try the next source.
    }
  }
  return 13
}

export async function initializeLogSettings() {
  if (!isTauri()) return
  let lines = await invoke<number | null>('get_log_max_lines')
  if (lines === null) {
    const legacy = maxLogLines(legacySlider())
    lines = await invoke<number>('set_log_max_lines', {
      lines: Number.isFinite(legacy) ? legacy : 0,
      migrateOnly: true,
    })
  }
  logSlider.value = sliderForLines(lines)
}

export function saveLogSettings() {
  clearTimeout(timer)
  timer = undefined
  if (!isTauri()) return
  const value = logSlider.value
  const lines = maxLogLines(value)
  logSaveError.value = ''
  pending = pending.then(async () => {
    try {
      await invoke('set_log_max_lines', {
        lines: Number.isFinite(lines) ? lines : 0,
        migrateOnly: false,
      })
    } catch (error) {
      if (logSlider.value === value) logSaveError.value = String(error)
    }
  })
}

export function updateLogSlider(value: number) {
  logSlider.value = value
  clearTimeout(timer)
  timer = setTimeout(saveLogSettings, 150)
}
