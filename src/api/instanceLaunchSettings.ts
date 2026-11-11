import { invoke } from '@tauri-apps/api/core'
import { createSettingsPersistence } from '@/util/settingsPersistence'

export interface InstanceLaunchSettings {
  title: string
  use_global_title: boolean
  disable_mod_update: boolean
  info: string
  server_login: number
  server_login_locked: boolean
  auth_server: string
  auth_register: string
  auth_name: string
  server_address: string
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
  title: '',
  use_global_title: false,
  disable_mod_update: false,
  info: '',
  server_login: 0,
  server_login_locked: false,
  auth_server: '',
  auth_register: '',
  auth_name: '',
  server_address: '',
  renderer: 0,
  prelaunch_command: '',
  prelaunch_wait: true,
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
