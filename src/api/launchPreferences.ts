import { invoke } from '@tauri-apps/api/core'

/** 全局启动偏好，对应 CE 的 `Config.Launch` 中可跨平台的部分。 */
export interface LaunchPreferences {
  /** 0 = 优先 IPv4，1 = JVM 默认，2 = 优先 IPv6。 */
  ip_stack: number
  prelaunch_command: string
  prelaunch_wait: boolean
  disable_jlw: boolean
  disable_legacy_fix: boolean
  disable_lwjgl_unsafe_agent: boolean
  lock_memory: boolean
  /** Windows 进程优先级：0 AboveNormal、1 Normal、2 BelowNormal、3 High、4 RealTime。 */
  process_priority: number
}

export const getLaunchPreferences = () => invoke<LaunchPreferences>('get_launch_preferences')

export const setLaunchPreferences = (preferences: LaunchPreferences) =>
  invoke<LaunchPreferences>('set_launch_preferences', { preferences })
