import { invoke } from '@tauri-apps/api/core'
import type { InstanceJavaSettings } from '@/api/javaReq'

export async function migrateLegacyInstanceJava() {
  const prefix = 'pcl-ui:PageInstanceSetup:'
  const legacy: Record<string, InstanceJavaSettings> = Object.create(null)
  for (let index = 0; index < localStorage.length; index++) {
    const key = localStorage.key(index)
    if (!key?.startsWith(prefix)) continue
    const id = key.slice(prefix.length)
    if (!id || id === 'none') continue
    try {
      const value = JSON.parse(localStorage.getItem(key) || '{}')
      const path = value?.ComboArgumentJava
      if (typeof path !== 'string' && typeof value?.CheckAdvanceJava !== 'boolean') continue
      legacy[id] = {
        preference:
          typeof path === 'string' && path !== '0' && path !== ''
            ? { kind: 'exist', java_exe: path }
            : { kind: 'auto' },
        ignore_compatibility: value?.CheckAdvanceJava === true,
      }
    } catch {
      /* Keep malformed legacy data intact. */
    }
  }
  return Object.keys(legacy).length
    ? invoke<string[]>('migrate_instance_java_settings', { legacy })
    : []
}
