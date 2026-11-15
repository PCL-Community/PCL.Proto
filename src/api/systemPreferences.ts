import { invoke } from '@tauri-apps/api/core'

/** 杂项页的启动器系统偏好。 */
export interface SystemPreferences {
  debug_mode: boolean
  random_delay: boolean
}

export const getSystemPreferences = () => invoke<SystemPreferences>('get_system_preferences')

export const setSystemPreferences = (preferences: SystemPreferences) =>
  invoke<SystemPreferences>('set_system_preferences', { preferences })
