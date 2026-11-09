import { isTauri } from '@tauri-apps/api/core'
import { arch, platform, type Platform } from '@tauri-apps/plugin-os'

const currentPlatform = isTauri() ? platform() : 'web'
export const currentArch = isTauri() ? arch() : ''
export default currentPlatform

/** 未指定平台表示所有平台通用；网页版仅用于展示，因此平台专属内容一律视为可用。 */
export function availableOnPlatform(target?: Platform): boolean {
  return !target || currentPlatform === 'web' || currentPlatform === target
}
