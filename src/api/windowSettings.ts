import { invoke } from '@tauri-apps/api/core'
import { createSettingsPersistence } from '@/util/settingsPersistence'

export interface WindowSettings {
  mode: 'default' | 'launcher' | 'custom'
  width: number
  height: number
}

export const windowSettingKeys = [
  'ComboArgumentWindowType',
  'TextArgumentWindowWidth',
  'TextArgumentWindowHeight',
]

export function legacyWindowSettings(): WindowSettings {
  let values: Record<string, unknown> = {}
  try {
    values = JSON.parse(localStorage.getItem('pcl-ui:PageSetupLaunch:global') || '{}') || {}
  } catch {
    /* Keep the legacy data intact. */
  }
  const dimension = (value: unknown, fallback: number) => {
    const number = typeof value === 'string' && /^\d+$/.test(value) ? Number(value) : NaN
    return Number.isInteger(number) && number >= 100 && number <= 16384 ? number : fallback
  }
  return {
    mode:
      values.ComboArgumentWindowType === '2'
        ? 'launcher'
        : values.ComboArgumentWindowType === '3'
          ? 'custom'
          : 'default',
    width: dimension(values.TextArgumentWindowWidth, 854),
    height: dimension(values.TextArgumentWindowHeight, 480),
  }
}

export async function initializeWindowSettings() {
  const saved = await invoke<WindowSettings | null>('get_window_settings')
  return (
    saved ||
    invoke<WindowSettings>('set_window_settings', {
      settings: legacyWindowSettings(),
      migrateOnly: true,
    })
  )
}

export const windowSettingsStorage = createSettingsPersistence(
  async () => {
    const settings = await invoke<WindowSettings | null>('get_window_settings')
    if (!settings) throw new Error('Window settings are not initialized')
    return settings
  },
  (_, settings) => invoke<WindowSettings>('set_window_settings', { settings, migrateOnly: false }),
)
