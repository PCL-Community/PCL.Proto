import { invoke } from '@tauri-apps/api/core'
import { createSettingsPersistence } from '@/util/settingsPersistence'

// Global 自定义信息: Minecraft shows it as `${version_type}`; empty drops the argument.
export const versionInfoStorage = createSettingsPersistence<string>(
  () => invoke<string>('get_version_info'),
  (_, value) => invoke<string>('set_version_info', { value }),
)
