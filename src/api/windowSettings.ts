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

const defaultWindowSettings: WindowSettings = { mode: 'default', width: 854, height: 480 }

export async function initializeWindowSettings() {
  return (
    (await invoke<WindowSettings | null>('get_window_settings')) ||
    invoke<WindowSettings>('set_window_settings', { settings: defaultWindowSettings })
  )
}

export const windowSettingsStorage = createSettingsPersistence(
  async () => {
    const settings = await invoke<WindowSettings | null>('get_window_settings')
    if (!settings) throw new Error('Window settings are not initialized')
    return settings
  },
  (_, settings) => invoke<WindowSettings>('set_window_settings', { settings }),
)
