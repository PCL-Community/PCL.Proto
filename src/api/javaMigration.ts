import { invoke } from '@tauri-apps/api/core'
import type { InstanceJavaSettings } from '@/api/javaReq'
import { dropLegacyFields, legacyInstancePages } from '@/util/legacySettings'

const legacyKeys = ['ComboArgumentJava', 'CheckAdvanceJava']

export async function migrateLegacyInstanceJava() {
  const legacy: Record<string, InstanceJavaSettings> = Object.create(null)
  const pages = new Map<string, string>()
  for (const { key, id, value } of legacyInstancePages('pcl-ui:PageInstanceSetup:')) {
    const path = value?.ComboArgumentJava
    if (typeof path !== 'string' && typeof value?.CheckAdvanceJava !== 'boolean') continue
    pages.set(id, key)
    legacy[id] = {
      preference:
        typeof path === 'string' && path !== '0' && path !== ''
          ? { kind: 'exist', java_exe: path }
          : { kind: 'auto' },
      ignore_compatibility: value?.CheckAdvanceJava === true,
    }
  }
  if (!Object.keys(legacy).length) return []
  const pending = await invoke<string[]>('migrate_instance_java_settings', { legacy })
  // Rust 已接管并记录迁移的实例，删除旧袋里的副本；pending 的保留供下次重试。
  for (const [id, key] of pages) if (!pending.includes(id)) dropLegacyFields(key, legacyKeys)
  return pending
}
