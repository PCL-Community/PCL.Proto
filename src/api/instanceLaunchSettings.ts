import { invoke } from '@tauri-apps/api/core'
import { createSettingsPersistence } from '@/util/settingsPersistence'

export interface InstanceLaunchSettings {
  info: string
  renderer: number
  prelaunch_command: string
  prelaunch_wait: boolean
  disable_asset_verify: boolean
  disable_jlw: boolean
  disable_legacy_fix: boolean
  debug_log4j2: boolean
  disable_lwjgl_unsafe_agent: boolean
  use_proxy: boolean
}

export const emptyInstanceLaunchSettings = (): InstanceLaunchSettings => ({
  info: '',
  renderer: 0,
  prelaunch_command: '',
  prelaunch_wait: false,
  disable_asset_verify: false,
  disable_jlw: false,
  disable_legacy_fix: false,
  debug_log4j2: false,
  disable_lwjgl_unsafe_agent: false,
  use_proxy: false,
})

export const instanceLaunchSettingsStorage = createSettingsPersistence(
  (instanceDirectory) =>
    invoke<InstanceLaunchSettings>('get_instance_launch_settings', { instanceDirectory }),
  (instanceDirectory, settings) =>
    invoke<InstanceLaunchSettings>('set_instance_launch_settings', { instanceDirectory, settings }),
)
